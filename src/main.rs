extern crate piston_app;
extern crate piston_window;
extern crate vecmath;

use piston_window as window;

use std::cmp;
use std::collections::BTreeMap;
use std::collections::HashMap;


pub mod prelude {
    pub type EID = usize;

    pub const NULL_ID: EID = EID::max_value();

    pub type Vec2 = ::vecmath::Vector2<f64>;
    pub use vecmath::{vec2_scale, vec2_add};
}

use prelude::*;


#[derive(Clone, Copy)]
enum Command {
    Nav(Vec2),
    Wait(f64),
    Shoot(EID),
}

#[derive(Clone, Copy)]
enum Weapon {
    Gun,
}

#[derive(Clone, Copy)]
enum Action {
    Mobile,
    Shoot,
    Dead,
}

#[derive(Clone, Copy)]
enum Effect {
    Die,
}

#[derive(Clone, Copy)]
struct UnitState {
    pos: Vec2,
    vel: Vec2,
    weapon: Weapon,
    action: Action,
    target_loc: Vec2,
    target_id: EID,

    time: f64,
    id: EID,
}

impl UnitState {
    fn update_pos(self: &mut Self, new_time: f64) {
        let vel = vec2_scale(self.vel, new_time - self.time);
        let pos = vec2_add(self.pos, vel);
        self.pos = pos;
        self.time = new_time;
    }
}

struct Time(f64);

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

type Timeline = BTreeMap<Time, Snapshot>;

#[derive(Clone)]
struct Snapshot {
    time: f64,
    states: HashMap<EID, UnitState>,
}

fn update_snapshot_precise(
    current: &mut Snapshot,
    timeline: &Timeline,
    new_time: f64,
) {
    let t1 = Time(current.time);
    let t2 = Time(new_time);
    for (_, units) in timeline.range(t1..t2) {
        for (&id, &unit) in &units.states {
            current.states.insert(id, unit);
        }
    }
    current.time = new_time;
}

fn update_snapshot(
    current: &mut Snapshot,
    timeline: &Timeline,
    new_time: f64,
) {
    update_snapshot_precise(current, timeline, new_time);
    for (_, unit) in &mut current.states {
        unit.update_pos(new_time);
    }
}

type Plan = HashMap<EID, Vec<Command>>;

struct ClientPlan {
    init: Snapshot,
    confirmed: Timeline,
    current: Snapshot,
    current_commands: HashMap<EID, Option<(f64, Command)>>,
    plans: Plan,
}

struct Client {
    client_a: ClientPlan,
    client_b: ClientPlan,
    display: Snapshot,
    planpaths: Plan,
    planned: Timeline,

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


struct Controls {
    select: window::Button,
    remove_comm: window::Button,
    nav: window::Button,
    shoot: window::Button,
    wait: window::Button,
    playpause: window::Button,
    restart: window::Button,
    switch_team: window::Button,
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
};

impl ClientPlan {
    fn new(init: Snapshot) -> Self {
        let confirmed = Timeline::new();
        let current = init.clone();
        let current_commands = empty_map(&init.states);
        let plans = empty_map(&init.states);
        ClientPlan { init, confirmed, current, current_commands, plans }
    }

    fn gen_planpaths(self: &Self) -> Plan {
        self.plans.clone()
    }

    fn gen_planned(self: &Self) -> (Plan, Timeline) {
        let planpaths = self.gen_planpaths();
        let mut sim = self.current.clone();
        let mut comm = self.current_commands.clone();
        let mut side_effects = Vec::new();
        let mut plans: HashMap<EID, _> = planpaths
            .iter()
            .map(|(&id, plan)| (id, plan.iter().peekable()))
            .collect();
        let mut timeline = Timeline::new();

        loop {
            let mut next: Option<(f64, EID)> = None;
            for (&id, plan) in &mut plans {
                if let Some((time, _)) = comm[&id] {
                    if next.is_none() || next.unwrap().0 > time {
                        next = Some((time, id));
                    }
                } else if plan.peek().is_some() {
                    next = Some((sim.time, id));
                }
            }
            if next.is_none() || next.unwrap().0 > sim.time {
                for (id, effect) in side_effects {
                    use self::Effect::*;
                    let state = sim.states.get_mut(&id).unwrap();
                    match effect {
                        Die => {
                            state.update_pos(sim.time);
                            state.action = Action::Dead;
                            state.vel = [0.0, 0.0];
                            state.target_id = NULL_ID;
                            state.target_loc = [0.0, 0.0];
                        },
                    }
                    let time = state.time;
                    timeline
                        .entry(Time(state.time))
                        .or_insert(Snapshot { time, states: HashMap::new() })
                        .states
                        .insert(id, *state);
                }
                side_effects = Vec::new();
            }
            if next.is_none() {
                break;
            }
            let (time, id) = next.unwrap();
            let mut state = sim.states[&id];
            let new_comm = plans.get_mut(&id).and_then(Iterator::next);
            let mut new_comm_time = time;
            sim.time = time;
            state.update_pos(time);
            match comm[&id] {
                None => (),
                Some((_, Command::Nav(_))) => {
                    state.vel = [0.0, 0.0];
                },
                Some((_, Command::Shoot(_))) => {
                    side_effects.push((state.target_id, Effect::Die));
                    state.action = Action::Mobile;
                    state.target_id = NULL_ID;
                    state.target_loc = [0.0, 0.0];
                },
                Some((_, Command::Wait(_))) => (),
            }
            match new_comm {
                None => (),
                Some(Command::Nav(pos)) => {
                    let disp = vecmath::vec2_sub(*pos, state.pos);
                    let max_speed = 1.0;
                    let min_duration = vecmath::vec2_len(disp) / max_speed;
                    let duration = min_duration.ceil();
                    new_comm_time += duration;
                    state.vel = vecmath::vec2_scale(disp, 1.0/duration);
                },
                Some(Command::Wait(duration)) => {
                    new_comm_time += duration;
                },
                Some(Command::Shoot(target)) => {
                    new_comm_time += 5.0;
                    state.target_id = *target;
                    state.action = Action::Shoot;
                },
            }
            sim.states.insert(id, state);
            let timeline_entry = timeline.entry(Time(time));
            timeline_entry
                .or_insert(Snapshot { time, states: HashMap::new() })
                .states
                .insert(id, state);
            comm.insert(id, new_comm.map(|comm| (new_comm_time, *comm)));
        }

        // (planpaths, timeline)
        (self.gen_planpaths(), timeline)
    }
}

impl Client {
    fn new(init: Snapshot) -> Self {
        let client_a = ClientPlan::new(init.clone());
        let client_b = ClientPlan::new(init);
        let display = client_a.init.clone();
        Client {
            client_a,
            client_b,
            display,
            planned: Timeline::new(),
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
        self.display = self.plan().current.clone();
        update_snapshot(&mut self.display, &self.planned, time);
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
        for (id, plan) in &self.planpaths {
            let mut pos = self.plan().current.states[id].pos;
            for command in plan {
                if let &Command::Nav(newpos) = command {
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
        }
        let client = &self.plan();
        for (&id, plan) in &client.plans {
            let rect = [-0.25, -0.25, 0.5, 0.5];
            let pos = client.current.states[&id].pos;
            let first_trans = trans.trans(pos[0], pos[1]);
            window::ellipse(path_color, rect, first_trans, graphics);

            for command in plan {
                if let &Command::Nav(pos) = command {
                    let trans = trans.trans(pos[0], pos[1]);
                    window::ellipse(path_color, rect, trans, graphics);
                }
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
        update_snapshot(&mut self.display, &self.planned, new_time);
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
                self.display = self.plan().current.clone();
            } else if args.button == CONTROLS.switch_team {
                self.display_a = !self.display_a;
                self.regen();
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
    let mut init = Snapshot {
        time: 0.0,
        states: HashMap::new(),
    };
    let killr = UnitState {
        id: 0,
        pos: [20.0, 30.0],
        vel: [0.0, 0.0],
        time: 0.0,

        weapon: Weapon::Gun,
        action: Action::Mobile,
        target_id: NULL_ID,
        target_loc: [0.0, 0.0],
    };
    let killd = UnitState {
        id: 1,
        pos: [40.0, 40.0],
        vel: [0.0, 0.0],
        time: 0.0,

        weapon: Weapon::Gun,
        action: Action::Mobile,
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
