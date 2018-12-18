use std::collections::HashMap;

use prelude::*;

use model;
use path;
use server::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Command {
    Nav(Vec2),
    Wait(f64),
    Shoot(EID),
}

impl model::UnitState {
    pub fn update_pos(self: &mut Self, new_time: f64) {
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

pub type Plan = HashMap<EID, Vec<Command>>;

pub struct Client {
    pub map: path::Map,
    pub mesh: path::NavMesh,

    pub init: model::Snapshot,
    pub confirmed: model::Timeline,
    pub current: model::Snapshot,
    pub current_commands: HashMap<EID, Option<(f64, Command)>>,
    pub plans: Plan,
}


impl Client {
    pub fn new(init: model::Snapshot, map: path::Map) -> Self {
        let mesh = path::NavMesh::generate(&map, 1.0);
        let confirmed = model::Timeline::new();
        let current = init.clone();
        let current_commands = empty_map(&init.states);
        let plans = empty_map(&init.states);
        Client {
            map,
            mesh,

            init,
            confirmed,
            current,
            current_commands,
            plans,
        }
    }

    fn gen_planpaths(self: &Self) -> Plan {
        self.plans.clone()
    }

    pub fn gen_planned(self: &Self) -> (Plan, model::Timeline) {
        let mut sims = Server::new(self.current.clone(), self.map.clone());

        let paths = self.gen_planpaths();
        // this is kinda dirty
        // I'm thinking about making a History struct and a Simulation struct,
        // but that doesn't really help here since we're dealing with commands
        let mut simc = Client {
            map: self.map.clone(),
            mesh: self.mesh.clone(),  // hmm...

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

    pub fn next_moves(self: &Self) -> HashMap<EID, model::UnitState> {
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

    pub fn accept_outcome(
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
