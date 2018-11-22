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
    Walk(Vec2),
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

type Timeline = BTreeMap<Time, UnitState>;

#[derive(Clone)]
struct Snapshot {
    time: f64,
    states: HashMap<EID, UnitState>,
}

struct Client {
    init: Snapshot,
    recent: Snapshot,
    display: Snapshot,
    confirmed: Timeline,
    planned: Timeline,
    plan: HashMap<EID, Vec<Command>>,
}

impl Client {
    fn new(init: Snapshot) -> Self {
        let recent = init.clone();
        let display = init.clone();
        let confirmed = BTreeMap::new();
        let planned = BTreeMap::new();
        let plan = init
            .states
            .iter()
            .map(|(&id, _)| (id, Vec::new()))
            .collect();
        Client { init, recent, display, confirmed, planned, plan }
    }
}

impl piston_app::App for Client {
    fn on_draw(
        self: &mut Self,
        centre: window::Context,
        graphics: &mut window::G2d,
        _args: window::RenderArgs,
    ) {
        window::clear([0.0, 0.0, 0.0, 1.0], graphics);

        let unit_color = [1.0, 1.0, 1.0, 1.0];
        let scale = 10.0;
        let trans = centre.transform;
        for (_, unit) in &self.display.states {
            let x = unit.pos[0] - 0.5;
            let y = unit.pos[1] - 0.5;
            let rect = [scale * x, scale * y, scale, scale];
            window::ellipse(unit_color, rect, trans, graphics);
        }
    }

    fn on_update(
        self: &mut Self,
        _args: window::UpdateArgs,
    ) {
        let dtime = 0.1;
        let new_time = self.display.time + dtime;
        for (_, unit) in &mut self.display.states {
            unit.update_pos(new_time);
        }
        let t1 = Time(self.display.time);
        let t2 = Time(new_time);
        for (_, unit) in self.planned.range(t1..t2) {
            let id = unit.id;
            let mut unit = *unit;
            unit.update_pos(new_time);
            self.display.states.insert(id, unit);
        }
        self.display.time = new_time;
    }
    fn on_input(
        self: &mut Self,
        _args: window::ButtonArgs,
    ) {
    }
    fn on_mouse_move(
        self: &mut Self,
        _mouse: [f64; 2],
    ) {
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
    let mut unit = UnitState {
        id: 0,
        pos: [30.0, 30.0],
        vel: [1.0, 0.0],
        time: 0.0,

        weapon: Weapon::Gun,
        action: Action::Mobile,
        target_id: NULL_ID,
        target_loc: [0.0, 0.0],
    };
    init.states.insert(0, unit);

    let mut client = Client::new(init);
    unit.time = 3.0;
    unit.pos = [33.0, 30.0];
    unit.vel = [1.0, -1.0];
    client.planned.insert(Time(unit.time), unit);
    piston_app::run_until_escape(client);
}
