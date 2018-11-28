use std::collections::BTreeMap;
use std::collections::HashMap;

use prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weapon {
    Gun,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Mobile,
    Shoot,
    Dead,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UnitState {
    pub pos: Vec2,
    pub vel: Vec2,
    pub weapon: Weapon,
    pub action: Action,
    pub target_loc: Vec2,
    pub target_id: EID,

    pub time: f64,
    pub id: EID,
}

pub struct Timeline {
    pub snapshots: BTreeMap<Time, Snapshot>,
}

impl Timeline {
    pub fn new() -> Self {
        Timeline { snapshots: BTreeMap::new() }
    }
    pub fn insert(self: &mut Self, state: UnitState) -> Option<UnitState> {
        let default_snap = Snapshot {
            time: state.time,
            states: HashMap::new(),
        };
        self.snapshots
            .entry(Time(state.time))
            .or_insert(default_snap)
            .insert(state)
    }
}

#[derive(Clone)]
pub struct Snapshot {
    pub time: f64,
    pub states: HashMap<EID, UnitState>,
}

impl Snapshot {
    pub fn insert(self: &mut Self, state: UnitState) -> Option<UnitState> {
        self.states
            .insert(state.id, state)
    }

    pub fn update_precise(
        self: &mut Snapshot,
        timeline: &Timeline,
        new_time: f64,
    ) {
        let t1 = Time(self.time);
        let t2 = Time(new_time);
        for (_, units) in timeline.snapshots.range(t1..t2) {
            for (&id, &unit) in &units.states {
                self.states.insert(id, unit);
            }
        }
        self.time = new_time;
    }

    pub fn update(
        self: &mut Snapshot,
        timeline: &Timeline,
        new_time: f64,
    ) {
        self.update_precise(timeline, new_time);
        for (_, unit) in &mut self.states {
            unit.update_pos(new_time);
        }
    }
}
