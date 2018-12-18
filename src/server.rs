use prelude::*;

use model;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Effect {
    Die,
}

pub struct Server {
    current: model::Snapshot,
}

impl Server {
    pub fn new(current: model::Snapshot) -> Self {
        Server { current }
    }

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

    pub fn resolve<I>(self: &mut Self, upd: I) -> Result<model::Snapshot, EID>
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
