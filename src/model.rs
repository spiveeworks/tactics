use std::collections::BTreeMap;
use std::collections::HashMap;
use std::iter;

use prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Weapon {
    Gun,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Action {
    Mobile,
    Shoot,
    Dead,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub snapshots: BTreeMap<Time, Snapshot>,
}

impl Timeline {
    pub fn new() -> Self {
        Timeline { snapshots: BTreeMap::new() }
    }
    pub fn insert(self: &mut Self, state: UnitState) -> Option<UnitState> {
        self.snapshots
            .entry(Time(state.time))
            .or_insert(Snapshot::with_time(state.time))
            .insert(state)
    }
    pub fn first(self: Self) -> Snapshot {
        self.snapshots
            .into_iter()
            .next()
            .map(|(_, x)| x)
            .unwrap_or(Snapshot::new())
    }
}

impl iter::FromIterator<UnitState> for Timeline {
    fn from_iter<I: IntoIterator<Item=UnitState>>(iter: I) -> Self {
        let mut result = Timeline::new();
        for x in iter {
            result.insert(x);
        }
        result
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub time: f64,
    pub states: HashMap<EID, UnitState>,
}

impl Snapshot {
    pub fn new() -> Self {
        Snapshot::with_time(0.0)
    }
    pub fn with_time(time: f64) -> Self {
        Snapshot {
            time,
            states: HashMap::new(),
        }
    }
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
        let range = ::std::ops::RangeInclusive::new(t1, t2);
        for (_, units) in timeline.snapshots.range(range) {
            self.copy_units(units);
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

    pub fn copy_units(
        self: &mut Snapshot,
        other: &Snapshot,
    ) {
        for (_, &unit) in &other.states {
            self.insert(unit);
        }
    }
}

impl iter::FromIterator<UnitState> for Snapshot {
    fn from_iter<I: IntoIterator<Item=UnitState>>(iter: I) -> Self {
        let mut result = Snapshot::new();
        for x in iter {
            if result.time < x.time {
                result.time = x.time;
            }
            if let Some(&old) = result.states.get(&x.id) {
                if old.time <= x.time {
                    result.insert(old);
                }
            }
        }
        result
    }
}

