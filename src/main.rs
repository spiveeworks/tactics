extern crate piston_app;
extern crate piston_window;
extern crate vecmath;

use piston_window as window;

use std::cmp;
use std::collections::BTreeSet;
use std::collections::HashMap;


pub mod prelude {
    pub type Vec2 = ::vecmath::Vector2<f64>;
    pub type EID = usize;

    pub const NULL_ID: EID = EID::max_value();
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

impl PartialEq for UnitState {
    fn eq(self: &Self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl Eq for UnitState {}

impl PartialOrd for UnitState {
    fn partial_cmp(self: &Self, other: &Self) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&self.time, &other.time)
    }
}

impl Ord for UnitState {
    fn cmp(self: &Self, other: &Self) -> cmp::Ordering {
        cmp::PartialOrd::partial_cmp(&self.time, &other.time)
            .expect("Got NaN as time...")
    }
}

type Timeline = BTreeSet<UnitState>;

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
        let confirmed = BTreeSet::new();
        let planned = BTreeSet::new();
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
        args: window::RenderArgs,
    ) {
    }

    fn on_update(
        self: &mut Self,
        args: window::UpdateArgs,
    ) {
    }
    fn on_input(
        self: &mut Self,
        args: window::ButtonArgs,
    ) {
    }
    fn on_mouse_move(
        self: &mut Self,
        mouse: [f64; 2],
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
    init.states.insert(0, UnitState {
        id: 0,
        pos: [0.0, 0.0],
        vel: [0.0, 0.0],
        time: 0.0,

        weapon: Weapon::Gun,
        action: Action::Mobile,
        target_id: NULL_ID,
        target_loc: [0.0, 0.0],
    });
    let client = Client::new(init);
    piston_app::run_until_escape(client);
}
