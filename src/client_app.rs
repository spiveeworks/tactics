
use piston_window as window;
use vecmath;
use piston_app;

use std::collections::HashMap;
use std::net;

use prelude::*;

use model;
use client::*;

pub struct ClientApp {
    client: Client,
    server: net::TcpStream,
    waiting: ServerState,

    display: model::Snapshot,
    updates: HashMap<EID, Update>,
    planpaths: Plan,
    planned: model::Timeline,

    selected: EID,
    mouse: Vec2,
    playing: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ServerState {
    Planning,
    ConfirmSubmit,
    Waiting,
    Display,
}

#[derive(Default, Clone, Copy)]
struct Update {
    vel: bool,
    target: bool,
}

impl Update {
    fn compare(one: model::UnitState, other: model::UnitState) -> Self {
        Update {
            vel: one.vel != other.vel,
            target: one.target_id != other.target_id,
        }
    }
}

struct Controls {
    select: window::Button,
    remove_comm: window::Button,
    cancel_comm: window::Button,
    nav: window::Button,
    shoot: window::Button,
    wait: window::Button,
    playpause: window::Button,
    restart: window::Button,
    submit: window::Button,
}

static CONTROLS: Controls = Controls {
    select:      window::Button::Mouse(window::mouse::MouseButton::Left),
    nav:         window::Button::Mouse(window::mouse::MouseButton::Right),
    remove_comm: window::Button::Keyboard(window::keyboard::Key::Backspace),
    cancel_comm: window::Button::Keyboard(window::keyboard::Key::X),
    shoot:       window::Button::Keyboard(window::keyboard::Key::Q),
    wait:        window::Button::Keyboard(window::keyboard::Key::W),
    playpause:   window::Button::Keyboard(window::keyboard::Key::Space),
    restart:     window::Button::Keyboard(window::keyboard::Key::R),
    submit:      window::Button::Keyboard(window::keyboard::Key::Return),
};

impl ClientApp {
    pub fn new<I: net::ToSocketAddrs>(ip: I, name: String) -> Self {
        //init: model::Snapshot, map: path::Map) -> Self {
        let server = net::TcpStream::connect(ip).expect("Failed to connect");
        ::bincode::serialize_into(&server, &name)
            .expect("Failed to send name to server");
        let map = ::bincode::deserialize_from(&server)
            .expect("Failed to download/parse map");
        let init = ::bincode::deserialize_from(&server)
            .expect("Failed to download/parse unit states");
        let intro: String = ::bincode::deserialize_from(&server)
            .expect("Failed to download roster");
        print!("{}", intro);
        let client = Client::new(init, map);

        let display = client.init.clone();
        let mut result = ClientApp {
            client,
            server,
            waiting: ServerState::Planning,

            display,
            updates: HashMap::new(),
            planned: model::Timeline::new(),
            planpaths: HashMap::new(),

            mouse: [0.0, 0.0],
            playing: false,
            selected: NULL_ID,
        };
        result.regen();
        result
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
            let mouse_id = if op == 3 {
                self.unit_nearest_mouse()
            } else {
                NULL_ID
            };
            let mouse = self.mouse;
            let plan = self.client.plans.get_mut(&id);
            if plan.is_none() {
                return;
            }
            let plan = plan.unwrap();
            match op {
                0 => {plan.pop();},
                1 => plan.push(Command::Cancel),
                2 => plan.push(Command::Nav(mouse)),
                3 => plan.push(Command::Shoot(mouse_id)),
                4 => plan.push(Command::Wait(1.0)),
                _ => panic!("edit.client called with {}", op),
            }
        }
        self.regen();
    }

    fn regen(self: &mut Self) {
        let time = self.display.time;
        self.regen_with_time(time);
    }

    fn regen_with_time(self: &mut Self, time: f64) {
        let (plan, timeline) = self.client.gen_planned();
        self.planpaths = plan;
        self.planned = timeline;
        self.display = self.get_display(time);
        let prevtime = time - 0.1;
        if prevtime >= self.client.init.time {
            let prevdisplay = self.get_display(prevtime);
            self.regen_updates(&prevdisplay);
        } else {
            self.updates = HashMap::new();
        }

    }

    fn regen_updates(self: &mut Self, old_states: &model::Snapshot) {
        self.updates = self
            .display
            .states
            .iter()
            .map(|(&id, &unit)|
                 (id, Update::compare(old_states.states[&id], unit))
             )
            .collect();
    }

    fn get_display(self: &Self, time: f64) -> model::Snapshot {
        let mut display;
        if time < self.client.current.time {
            display = self.client.init.clone();
            display.update(&self.client.confirmed, time);
        } else {
            display = self.client.current.clone();
            display.update(&self.planned, time);
        }
        display
    }

    fn submit_server(self: &mut Self) {
        let plan = self.client.next_moves();
        ::bincode::serialize_into(&self.server, &plan)
            .expect("Failed to send plan to server");
        self.waiting = ServerState::Waiting;
    }

    fn check_server(self: &mut Self) {
        if self.waiting != ServerState::Waiting {
            return;
        }
        let mut byte = [0];
        let success = self.server.peek(&mut byte);
        if success.is_err() || success.unwrap() != 1 {
            return;
        }
        let result = ::bincode::deserialize_from(&self.server)
            .expect("Failed to receive result from server");
        self.waiting = ServerState::Display;
        let plan = self.client.next_moves();
        self.client.accept_outcome(&plan, &result);
        self.regen_with_time(result.time);
        self.playing = false;
    }
}

static SCALE: f64 = 10.0;

fn dotted_line(
    col: [f32; 4],
    radius: f64,
    //on_len: f64,
    //off_len: f64,
    //offset: f64,
    line: [f64; 4],
    trans: window::math::Matrix2d,
    graphics: &mut window::G2d,
) {
    let mut pos = [line[0], line[1]];
    let dest = [line[2], line[3]];
    let mut alternate = true;
    let dir = vecmath::vec2_normalized(vec2_sub(dest, pos));
    let len = 1.0;
    while pos != dest {
        let diff = vec2_sub(dest, pos);
        let next_pos;
        if vecmath::vec2_square_len(diff) < len * len {
            next_pos = dest;
        } else {
            next_pos = vec2_add(pos, vec2_scale(dir, len));
        }
        if alternate {
            let segment = [pos[0], pos[1], next_pos[0], next_pos[1]];
            window::line(col, radius, segment, trans, graphics);
        }
        alternate = !alternate;
        pos = next_pos;
    }
}

fn sys_time() -> f64 {
    use std::time::SystemTime;
    let now = SystemTime::now();
    let now = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let now_nano = now.subsec_nanos() as f64 / 1_000_000_000.0;
    let now_sec = now.as_secs() as f64;
    now_sec + now_nano
}

impl piston_app::App for ClientApp {
    fn on_draw(
        self: &mut Self,
        centre: window::Context,
        graphics: &mut window::G2d,
        _args: window::RenderArgs,
    ) {
        use piston_window::Transformed;
        use self::window::Graphics;
        window::clear([0.0, 0.0, 0.0, 1.0], graphics);

        let flash_speed = 0.5;
        let flash_new = self.playing ||
            sys_time() % (2.0*flash_speed) < flash_speed;

        let unit_color = [1.0, 1.0, 1.0, 1.0];
        let scale = SCALE;
        let trans = centre.transform.scale(scale, scale);

        let unit_shape = [-0.5, -0.5, 1.0, 1.0];

        let vel_pos = 0.7;
        let vel_size = 0.3;
        let vel_shape = [
            [vel_pos, vel_size],
            [vel_pos + vel_size, 0.0],
            [vel_pos, -vel_size]
        ];
        for (&id, &unit) in &self.display.states {
            let unit_trans = trans.trans(unit.pos[0], unit.pos[1]);

            let updates = self.updates.get(&id).cloned().unwrap_or_default();
            if flash_new || !(unit.vel == [0.0;2] && updates.vel) {
                window::ellipse(unit_color, unit_shape, unit_trans, graphics);
            }

            if unit.target_id != NULL_ID && (flash_new || !updates.target) {
                let pos = self.display.states[&unit.target_id].pos;
                let line = [unit.pos[0], unit.pos[1], pos[0], pos[1]];
                dotted_line(unit_color, 1.0/scale, line, trans, graphics);
            }

            if unit.vel != [0.0; 2] && (flash_new || !updates.vel) {
                let vel_trans = unit_trans.orient(unit.vel[0], unit.vel[1]);
                let mut tri = [[0.0;2];3];
                for i in 0..3 {
                    let x = vel_shape[i][0];
                    let y = vel_shape[i][1];
                    tri[i][0] = window::triangulation::tx(vel_trans, x, y);
                    tri[i][1] = window::triangulation::ty(vel_trans, x, y);
                }
                graphics.tri_list(
                    &Default::default(),
                    &unit_color,
                    |f| f(&tri),
                );
            }
        }

        let path_color = [1.0, 1.0, 1.0, 1.0];
        let client = &self.client;
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

        let mut tri_list = Vec::with_capacity(client.map.len() * 3);
        for &trig in &self.client.map {
            // rust unroll plz <3
            for p in 0..3 {
                let x = trig[p][0];
                let y = trig[p][1];
                // surely im not meant to be doing this
                let tx = window::triangulation::tx(trans, x, y);
                let ty = window::triangulation::ty(trans, x, y);
                tri_list.push([tx, ty]);
            }
        }
        graphics.tri_list(&Default::default(), &path_color, |f| f(&*tri_list));

        use self::ServerState::*;
        match self.waiting {
            Planning => (),
            ConfirmSubmit => window::ellipse(
                [1.0,1.0,0.0,1.0],
                [5.0,5.0,15.0,15.0],
                centre.transform,
                graphics,
            ),
            Waiting => window::ellipse(
                [1.0,0.0,0.0,1.0],
                [5.0,5.0,15.0,15.0],
                centre.transform,
                graphics,
            ),
            Display => window::ellipse(
                [0.0,1.0,0.0,1.0],
                [5.0,5.0,15.0,15.0],
                centre.transform,
                graphics,
            ),
        }
    }

    fn on_update(
        self: &mut Self,
        _args: window::UpdateArgs,
    ) {
        self.check_server();
        if !self.playing {
            return
        }
        let dtime = 0.1;
        let new_time = self.display.time + dtime;
        let old_states = self.display.clone();
        {
            let tl;
            // redundancy because borrow checker
            if new_time <= self.client.current.time {
                tl = &self.client.confirmed;
            } else {
                if self.display.time <= self.client.current.time {
                    self.display = self.client.current.clone();
                }
                tl = &self.planned;
            }
            self.display.update(tl, new_time);
        }
        self.regen_updates(&old_states);
    }
    fn on_input(
        self: &mut Self,
        args: window::ButtonArgs,
    ) {
        if args.state == window::ButtonState::Press {
            let mut input = true;
            if args.button == CONTROLS.select {
                self.selected = self.unit_nearest_mouse();
            } else if args.button == CONTROLS.remove_comm {
                self.edit_plan(0);
            } else if args.button == CONTROLS.cancel_comm {
                self.edit_plan(1);
            } else if args.button == CONTROLS.nav {
                self.edit_plan(2);
            } else if args.button == CONTROLS.shoot {
                self.edit_plan(3);
            } else if args.button == CONTROLS.wait {
                self.edit_plan(4);
            } else if args.button == CONTROLS.playpause {
                self.playing = !self.playing;
            } else if args.button == CONTROLS.restart {
                let dt = self.display.time;
                let ct = self.client.current.time;
                let new_dt;
                if dt > ct || dt == self.client.init.time {
                    new_dt = self.client.current.time;
                } else {
                    new_dt = self.client.init.time;
                }
                self.regen_with_time(new_dt);
            } else if args.button == CONTROLS.submit {
                use self::ServerState::*;
                match self.waiting {
                    Planning | Display => self.waiting = ConfirmSubmit,
                    ConfirmSubmit => self.submit_server(),
                    Waiting => (),
                }
                input = false;
            }
            if input && self.waiting != ServerState::Waiting {
                self.waiting = ServerState::Planning;
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

