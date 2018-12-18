
use piston_window as window;

use std::collections::HashMap;

use prelude::*;

use model;
use client::*;
use server::*;

pub struct ClientApp {
    client_a: Client,
    client_b: Client,
    server: Server,
    display: model::Snapshot,
    planpaths: Plan,
    planned: model::Timeline,

    selected: EID,
    mouse: Vec2,
    playing: bool,
    display_a: bool,
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

impl ClientApp {
    fn new(init: model::Snapshot) -> Self {
        let client_a = Client::new(init.clone());
        let client_b = Client::new(init);
        let server = Server::new(client_a.current.clone());

        let display = client_a.init.clone();
        ClientApp {
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

    fn plan(self: &Self) -> &Client {
        if self.display_a {
            &self.client_a
        } else {
            &self.client_b
        }
    }

    fn plan_mut(self: &mut Self) -> &mut Client {
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

    pub fn new_demo() -> Self {
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

        let mut client = ClientApp::new(init);
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
        client
    }
}

static SCALE: f64 = 10.0;

impl piston_app::App for ClientApp {
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

