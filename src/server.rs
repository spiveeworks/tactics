use prelude::*;

use model;
use path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Effect {
    Die,
    StopMoving,
    StopShooting,
}

pub struct Server {
    current: model::Snapshot,
    map: path::Map,
}

impl Server {
    pub fn new(current: model::Snapshot, map: path::Map) -> Self {
        Server { current, map }
    }

    fn consequence(self: &Self, next: f64) -> Vec<(f64, EID, Effect)> {
        let mut result = Vec::new();
        result.extend(self.consequence_tools());
        result.extend(self.consequence_boundaries(next));
        result
    }

    fn consequence_tools(self: &Self) -> Vec<(f64, EID, Effect)> {
        let mut result = Vec::new();
        for (_, &unit) in &self.current.states {
            if unit.action == model::Action::Shoot {
                let target = unit.target_id;
                let curr = self.current.states[&target];
                if curr.action != model::Action::Dead {
                    result.push((unit.time + 5.0, target, Effect::Die)); }
            }
        }
        result
    }

    fn consequence_boundaries(self: &Self, next: f64)
        -> Vec<(f64, EID, Effect)>
    {
        let mut result = Vec::new();
        let future = model::Timeline::new(); // the future is unknown :)
        let mut time = self.current.time;
        while result.len() == 0 && time < next + 0.1 {
            let mut state = self.current.clone();
            time += 0.1;
            state.update(&future, time);
            for (&id, &unit) in &state.states {
                let eff = Self::unit_collision(&self.map, &state, unit);
                if let Some((eff_time, effect)) = eff {
                    result.push((eff_time, id, effect));
                }
            }
        }
        result
    }

    fn unit_collision(
        map: &path::Map,
        state: &model::Snapshot,
        unit: model::UnitState,
    ) -> Option<(f64, Effect)> {
        if unit.vel != [0.0; 2] {
            if path::unit_is_in_wall(&map, unit.pos) {
                return Some((unit.time - 0.1, Effect::StopMoving));
            }
        }
        use model::Action::*;
        match unit.action {
            Mobile | Dead => (),
            Shoot => {
                let target = state.states[&unit.target_id].pos;
                let can_see = path::unit_can_see_pos(
                    &map,
                    unit.pos,
                    target,
                );
                if !can_see {
                    return Some((unit.time, Effect::StopShooting));
                }
            },
        }
        None
    }

    pub fn resolve<I>(self: &mut Self, upd: I) -> Result<model::Snapshot, EID>
        where I: Iterator<Item = model::UnitState>
    {
        let mut sorted_states = model::Timeline::new();
        for unit in upd {
            if !self.is_valid(unit) {
                return Err(unit.id);
            }
            sorted_states.insert(unit);
        }
        let mut snap = sorted_states.first();

        let ext = self.consequence(snap.time);

        // would be a lot clearer if we just used the old ```
        //   time = min(snap.time, ext.times().min())
        //   if ext.times.min < time { apply ext }
        // ```
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
        if Self::unit_collision(&self.map, &self.current, unit).is_some() {
            return false;
        }
        // also position continuity,
        // action state flow,
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
            StopMoving | StopShooting => {
                if state.time < time {
                    state.update_pos(time);
                    match effect {
                        StopMoving => {
                            state.vel = [0.0, 0.0];
                        },
                        StopShooting => {
                            state.target_id = NULL_ID;
                            state.target_loc = [0.0, 0.0];
                        },
                        Die => unreachable!(),
                    }
                }
            },
        }
    }
}
