extern crate piston_app;
extern crate piston_window;
extern crate vecmath;

use piston_window as window;

use std::collections::HashMap;

mod model;

pub mod prelude {
    pub type EID = usize;

    pub const NULL_ID: EID = EID::max_value();

    pub type Vec2 = ::vecmath::Vector2<f64>;
    pub use vecmath::{vec2_scale, vec2_add};

    // should use NotNaN crate
    #[derive(Clone, Copy)]
    pub struct Time(pub f64);

    use std::cmp;
    impl PartialEq for Time {
        fn eq(self: &Self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    impl Eq for Time {}

    impl PartialOrd for Time {
        fn partial_cmp(self: &Self, other: &Self) -> Option<cmp::Ordering> {
            PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }

    impl Ord for Time {
        fn cmp(self: &Self, other: &Self) -> cmp::Ordering {
            cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                .expect("Got NaN as time...")
        }
    }
}

use prelude::*;


#[derive(Clone, Copy, PartialEq, Debug)]
enum Command {
    Nav(Vec2),
    Wait(f64),
    Shoot(EID),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Effect {
    Die,
}

impl model::UnitState {
    fn update_pos(self: &mut Self, new_time: f64) {
        let vel = vec2_scale(self.vel, new_time - self.time);
        let pos = vec2_add(self.pos, vel);
        self.pos = pos;
        self.time = new_time;
    }

    fn command_end(self: &mut Self, comm: Command, time: f64) {
        self.update_pos(time);
        match comm {
            Command::Nav(_) => {
                self.vel = [0.0, 0.0];
            },
            Command::Shoot(_) => {
                self.action = model::Action::Mobile; self.target_id = NULL_ID;
                self.target_loc = [0.0, 0.0];
            },
            Command::Wait(_) => (),
        }
    }

    fn command_start(self: &mut Self, comm: Command) -> f64 {
        let duration = self.command_duration(comm);
        match comm {
            Command::Nav(pos) => {
                let disp = vecmath::vec2_sub(pos, self.pos);
                self.vel = vecmath::vec2_scale(disp, 1.0/duration);
            },
            Command::Shoot(target) => {
                self.target_id = target;
                self.action = model::Action::Shoot;
            },
            Command::Wait(_) => (),
        }
        duration
    }

    fn walk_duration(self: &Self, pos: Vec2) -> f64 {
        let disp = vecmath::vec2_sub(pos, self.pos);
        let max_speed = 1.0;
        let min_duration = vecmath::vec2_len(disp) / max_speed;
        let duration = (min_duration*10.0).ceil()*0.1;
        // prevents NaN, but 0-length commands currently cause problems anyway
        if duration == 0.0 {
            0.1
        } else {
            duration
        }
    }

    fn command_duration(self: &Self, comm: Command) -> f64 {
        match comm {
            Command::Nav(pos) => {
                self.walk_duration(pos)
            },
            Command::Wait(duration) => {
                duration
            },
            Command::Shoot(_) => {
                5.0
            },
        }
    }

    // infers a command that would start with the given unit state, and might
    // finish at the given time
    fn infer_command(self: Self, finish: f64) -> Option<(f64, Command)> {
        use model::Action::*;
        match self.action {
            Shoot => Some((self.time + 5.0, Command::Shoot(self.target_id))),
            Mobile => if self.vel == [0.0, 0.0] {
                None
            } else {
                let mut dummy = self;
                dummy.update_pos(finish);
                Some((finish, Command::Nav(dummy.pos)))
            },
            _ => None,
        }
    }
}

type Plan = HashMap<EID, Vec<Command>>;

struct Server {
    current: model::Snapshot,
}

struct ClientPlan {
    init: model::Snapshot,
    confirmed: model::Timeline,
    current: model::Snapshot,
    current_commands: HashMap<EID, Option<(f64, Command)>>,
    plans: Plan,
}

struct Client {
    client_a: ClientPlan,
    client_b: ClientPlan,
    server: Server,
    display: model::Snapshot,
    planpaths: Plan,
    planned: model::Timeline,

    selected: EID,
    mouse: Vec2,
    playing: bool,
    display_a: bool,
}

fn empty_map<T, U: Default>(base: &HashMap<EID, T>) -> HashMap<EID, U> {
    base.iter()
        .map(|(&id, _)| (id, Default::default()))
        .collect()
}

impl Server {
    fn consequence(self: &Self) -> Vec<(f64, EID, Effect)> {
        let mut result = Vec::new();
        for (_, &unit) in &self.current.states {
            if unit.action == model::Action::Shoot {
                let target = unit.target_id;
                let curr = self.current.states[&target];
                if curr.action != model::Action::Dead {
                    result.push((unit.time + 5.0, target, Effect::Die));
                }
            }
        }
        result
    }

    fn resolve<I>(self: &mut Self, upd: I) -> Result<model::Snapshot, EID>
        where I: Iterator<Item = model::UnitState>
    {
        let ext = self.consequence();

        let mut sorted_states = model::Timeline::new();
        for unit in upd {
            if !self.is_valid(unit) {
                return Err(unit.id);
            }
            sorted_states.insert(unit);
        }
        let mut snap = sorted_states.first();

        for (time, id, effect) in ext {
            if snap.states.len() == 0 || snap.time > time {
                snap = model::Snapshot::with_time(time);
            }
            if snap.time == time {
                let mut state = snap
                    .states
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| self.current.states[&id]);
                Self::apply_effects(
                    &mut state,
                    time,
                    effect,
                );
                snap.insert(state);
            }
        }

        // else its time field is probably 0.0 which we might not want
        if snap.states.len() > 0 {
            self.current.time = snap.time;
            self.current.copy_units(&snap);
        }

        Ok(snap)
    }

    fn is_valid(self: &Self, unit: model::UnitState) -> bool {
        true
    }

    fn apply_effects(
        state: &mut model::UnitState,
        time: f64,
        effect: Effect,
    ) {
        use self::Effect::*;
        match effect {
            Die => {
                state.update_pos(time);
                state.action = model::Action::Dead;
                state.vel = [0.0, 0.0];
                state.target_id = NULL_ID;
                state.target_loc = [0.0, 0.0];
            },
        }
    }
}

struct Controls {
    select: window::Button,
    remove_comm: window::Button,
    nav: window::Button,
    shoot: window::Button,
    wait: window::Button,
    playpause: window::Button,
    restart: window::Button,
    switch_team: window::Button,
    submit: window::Button,
}

static CONTROLS: Controls = Controls {
    select:      window::Button::Mouse(window::mouse::MouseButton::Left),
    nav:         window::Button::Mouse(window::mouse::MouseButton::Right),
    remove_comm: window::Button::Keyboard(window::keyboard::Key::Backspace),
    shoot:       window::Button::Keyboard(window::keyboard::Key::Q),
    wait:        window::Button::Keyboard(window::keyboard::Key::W),
    playpause:   window::Button::Keyboard(window::keyboard::Key::Space),
    restart:     window::Button::Keyboard(window::keyboard::Key::R),
    switch_team: window::Button::Keyboard(window::keyboard::Key::LAlt),
    submit:      window::Button::Keyboard(window::keyboard::Key::Return),
};

impl ClientPlan {
    fn new(init: model::Snapshot) -> Self {
        let confirmed = model::Timeline::new();
        let current = init.clone();
        let current_commands = empty_map(&init.states);
        let plans = empty_map(&init.states);
        ClientPlan { init, confirmed, current, current_commands, plans }
    }

    fn gen_planpaths(self: &Self) -> Plan {
        self.plans.clone()
    }

    fn gen_planned(self: &Self) -> (Plan, model::Timeline) {
        let mut sims = Server {
            current: self.current.clone(),
        };
        let paths = self.gen_planpaths();
        // this is kinda dirty
        // I'm thinking about making a History struct and a Simulation struct,
        // but that doesn't really help here since we're dealing with commands
        let mut simc = ClientPlan {
            init: model::Snapshot::new(),
            confirmed: model::Timeline::new(),
            current_commands: self.current_commands.clone(),
            current: self.current.clone(),
            plans: paths.clone(),
        };

        loop {
            let next = simc.next_moves();
            let result = sims.resolve(
                next.iter()
                    .map(|(_, &unit)| unit)
            ).unwrap();
            if result.states.len() == 0 {
                break;
            }
            simc.accept_outcome(&next, &result);
        }

        (paths, simc.confirmed)
    }

    fn next_moves(self: &Self) -> HashMap<EID, model::UnitState> {
        let mut moves = HashMap::new();
        for (&id, &comm) in &self.current_commands {
            let old_state = self.current.states[&id];
            let mut state = old_state;
            let new_comm = self
                .plans
                .get(&id)
                .and_then(|x| x.get(0))
                .cloned();
            if let Some((time, comm)) = comm {
                state.command_end(comm, time);
            }
            if let Some(new_comm) = new_comm {
                state.command_start(new_comm);
            }
            if state != old_state {
                moves.insert(id, state);
            }
        }
        moves
    }

    fn accept_outcome(
        self: &mut Self,
        expected: &HashMap<EID, model::UnitState>,
        outcome: &model::Snapshot,
    ) {
        // TODO figure out a consistent way of dealing with the 0.1 buffer
        self.current.time = outcome.time + 0.1;
        self.confirmed.snapshots.insert(Time(outcome.time), outcome.clone());
        for (&id, &unit) in &outcome.states {
            self.current.states.insert(id, unit);
            let mut expected = expected
                .get(&id)
                .cloned()
                .unwrap_or(self.current.states[&id]);
            if unit == expected {
                let comm = if self.plans[&id].len() > 0 {
                    Some(self.plans.get_mut(&id).unwrap().remove(0))
                } else {
                    None
                };
                let comm = comm.map(|c|
                    (unit.time + unit.command_duration(c), c)
                );
                self.current_commands.insert(id, comm);
            } else {
                self.plans.insert(id, Vec::new());
                let comm = unit.infer_command(self.current.time);
                self.current_commands.insert(id, comm);
            }
        }
    }
}

impl Client {
    fn new(init: model::Snapshot) -> Self {
        let client_a = ClientPlan::new(init.clone());
        let client_b = ClientPlan::new(init);
        let server = Server {
            current: client_a.current.clone(),
        };
        let display = client_a.init.clone();
        Client {
            client_a,
            client_b,
            server,
            display,
            planned: model::Timeline::new(),
            planpaths: HashMap::new(),

            mouse: [0.0, 0.0],
            playing: false,
            display_a: true,
            selected: NULL_ID,
        }
    }

    fn plan(self: &Self) -> &ClientPlan {
        if self.display_a {
            &self.client_a
        } else {
            &self.client_b
        }
    }

    fn plan_mut(self: &mut Self) -> &mut ClientPlan {
        if self.display_a {
            &mut self.client_a
        } else {
            &mut self.client_b
        }
    }

    fn unit_nearest_mouse(self: &Self) -> EID {
        let mut select = NULL_ID;
        let mut select_dist = 0.0;
        for (&id, val) in &self.display.states {
            let disp = vecmath::vec2_sub(val.pos, self.mouse);
            let dist = vecmath::vec2_len(disp);
            if select == NULL_ID || dist < select_dist {
                select = id;
                select_dist = dist;
            }
        }
        return select;
    }

    fn edit_plan(self: &mut Self, op: u16) {
        {
            let id = self.selected;
            let mouse_id = if op == 2 {
                self.unit_nearest_mouse()
            } else {
                NULL_ID
            };
            let mouse = self.mouse;
            let plan = self.plan_mut().plans.get_mut(&id);
            if plan.is_none() {
                return;
            }
            let plan = plan.unwrap();
            match op {
                0 => {plan.pop();},
                1 => plan.push(Command::Nav(mouse)),
                2 => plan.push(Command::Shoot(mouse_id)),
                3 => plan.push(Command::Wait(1.0)),
                _ => panic!("edit_plan() called with {}", op),
            }
        }
        self.regen();
    }

    fn regen(self: &mut Self) {
        let (plan, timeline) = self.plan().gen_planned();
        self.planpaths = plan;
        self.planned = timeline;
        let time = self.display.time;
        if time < self.plan().current.time {
            self.display = self.plan().init.clone();
            if self.display_a {
                self.display.update(&self.client_a.confirmed, time);
            } else {
                self.display.update(&self.client_b.confirmed, time);
            }
        } else {
            self.display = self.plan().current.clone();
            self.display.update(&self.planned, time);
        }
    }

    fn submit_server(self: &mut Self) {
        let lplan = self.client_a.next_moves();
        let rplan = self.client_b.next_moves();
        let mut moves = Vec::new();
        if let Some(&next_move) = lplan.get(&0) {
            moves.push(next_move);
        }
        if let Some(&next_move) = rplan.get(&1) {
            moves.push(next_move);
        }
        let result = self
            .server
            .resolve(moves.into_iter())
            .expect("Server rejected plan");
        self.client_a.accept_outcome(&lplan, &result);
        self.client_b.accept_outcome(&rplan, &result);
        self.regen();
    }
}

static SCALE: f64 = 10.0;

impl piston_app::App for Client {
    fn on_draw(
        self: &mut Self,
        centre: window::Context,
        graphics: &mut window::G2d,
        _args: window::RenderArgs,
    ) {
        use piston_window::Transformed;
        window::clear([0.0, 0.0, 0.0, 1.0], graphics);

        let unit_color = [1.0, 1.0, 1.0, 1.0];
        let scale = SCALE;
        let trans = centre.transform.scale(scale, scale);
        for (_, unit) in &self.display.states {
            let x = unit.pos[0] - 0.5;
            let y = unit.pos[1] - 0.5;
            let rect = [x, y, 1.0, 1.0];
            window::ellipse(unit_color, rect, trans, graphics);
        }

        let path_color = [1.0, 1.0, 1.0, 1.0];
        let client = &self.plan();
        for (id, plan) in &self.planpaths {
            let mut pos_list = Vec::new();
            let mut unit = client.current.states[&id];
            unit.update_pos(client.current.time);
            let mut pos = unit.pos;

            if let Some((_, Command::Nav(pos))) = client.current_commands[&id]
            {
                pos_list.push(pos);
            }
            for command in plan {
                if let &Command::Nav(pos) = command {
                    pos_list.push(pos);
                }
            }
            for newpos in pos_list {
                let line = [
                    pos[0],
                    pos[1],
                    newpos[0],
                    newpos[1]
                ];
                let r = 1.0/scale;
                window::line(path_color, r, line, trans, graphics);
                pos = newpos;
            }
        }
        for (&id, plan) in &client.plans {
            let rect = [-0.25, -0.25, 0.5, 0.5];
            let mut pos_list = Vec::new();
            let mut unit = client.current.states[&id];
            unit.update_pos(client.current.time);
            pos_list.push(unit.pos);

            if let Some((_, Command::Nav(pos))) = client.current_commands[&id]
            {
                pos_list.push(pos);
            }
            for command in plan {
                if let &Command::Nav(pos) = command {
                    pos_list.push(pos);
                }
            }
            for pos in pos_list {
                let trans = trans.trans(pos[0], pos[1]);
                window::ellipse(path_color, rect, trans, graphics);
            }
        }
    }

    fn on_update(
        self: &mut Self,
        _args: window::UpdateArgs,
    ) {
        if !self.playing {
            return
        }
        let dtime = 0.1;
        let new_time = self.display.time + dtime;
        let tl;
        // redundancy because borrow checker
        if new_time <= self.plan().current.time {
            tl = if self.display_a {
                &self.client_a.confirmed
            } else {
                &self.client_b.confirmed
            }
        } else {
            if self.display.time <= self.plan().current.time {
                self.display = self.plan().current.clone();
            }
            tl = &self.planned;
        }
        self.display.update(tl, new_time);
    }
    fn on_input(
        self: &mut Self,
        args: window::ButtonArgs,
    ) {
        if args.state == window::ButtonState::Press {
            if args.button == CONTROLS.select {
                self.selected = self.unit_nearest_mouse();
            } else if args.button == CONTROLS.remove_comm {
                self.edit_plan(0);
            } else if args.button == CONTROLS.nav {
                self.edit_plan(1);
            } else if args.button == CONTROLS.shoot {
                self.edit_plan(2);
            } else if args.button == CONTROLS.wait {
                self.edit_plan(3);
            } else if args.button == CONTROLS.playpause {
                self.playing = !self.playing;
            } else if args.button == CONTROLS.restart {
                let dt = self.display.time;
                let ct = self.plan().current.time;
                if dt > ct || dt == 0.0 {
                    self.display = self.plan().current.clone();
                } else {
                    self.display = self.plan().init.clone();
                }
            } else if args.button == CONTROLS.switch_team {
                self.display_a = !self.display_a;
                self.regen();
            } else if args.button == CONTROLS.submit {
                self.submit_server();
            }
        }
    }
    fn on_mouse_move(
        self: &mut Self,
        mouse: [f64; 2],
    ) {
        self.mouse = vecmath::vec2_scale(mouse, 1.0/SCALE);
    }

    fn window_name() -> &'static str {
        "perfect information tactical planning (prototype)"
    }
    fn window_starting_size() -> [u32; 2] {
        [600, 600]
    }
}


fn main() {
    let mut init = model::Snapshot {
        time: 0.0,
        states: HashMap::new(),
    };
    let killr = model::UnitState {
        id: 0,
        pos: [20.0, 30.0],
        vel: [0.0, 0.0],
        time: 0.0,

        weapon: model::Weapon::Gun,
        action: model::Action::Mobile,
        target_id: NULL_ID,
        target_loc: [0.0, 0.0],
    };
    let killd = model::UnitState {
        id: 1,
        pos: [40.0, 40.0],
        vel: [0.0, 0.0],
        time: 0.0,

        weapon: model::Weapon::Gun,
        action: model::Action::Mobile,
        target_id: NULL_ID,
        target_loc: [0.0, 0.0],
    };
    init.states.insert(0, killr);
    init.states.insert(1, killd);

    let mut client = Client::new(init);
    {
        let plan = [
            (0, Command::Shoot(1)),
            (1, Command::Nav([40.0, 20.0])),
        ];
        for &(id, com) in &plan {
            client
                .plan_mut()
                .plans
                .get_mut(&id)
                .unwrap()
                .push(com);
        }
    }
    client.regen();
    piston_app::run_until_escape(client);
}
