use std::collections::HashMap;

use prelude::*;

use model;
use path;

#[derive(Serialize, Deserialize)]
struct Unit {
    team: TID,
    pos: (f64, f64),
    weapon: model::Weapon,
}
#[derive(Serialize, Deserialize)]
struct Scenario {
    units: Vec<Unit>,
    map: Vec<Vec<Vec2>>,
}

pub fn read_scenario(path: &String)
    -> (HashMap<EID, TID>, model::Snapshot, path::Map)
{
    let mut file = ::std::fs::File::open(path)
        .expect("Couldn't open file");
    let mut stuff = String::new();
    use std::io::Read;
    file.read_to_string(&mut stuff)
        .expect("Couldn't read file");

    let Scenario { units, map } = ::ron::de::from_str(&stuff)
        .expect("Failed to read file");

    let (teams, init) = read_units(units);
    let map = read_map(map);

    (teams, init, map)
}

fn read_units(units: Vec<Unit>) -> (HashMap<EID, TID>, model::Snapshot) {
    let mut teams = HashMap::new();
    let mut init = model::Snapshot::new();
    for i in 0..units.len() {
        let id = i as EID;
        let Unit { pos: (x, y), team, weapon } = units[i];
        let unit = model::UnitState {
            id,
            pos: [x, y],
            vel: [0.0, 0.0],
            time: init.time,

            weapon,
            action: model::Action::Mobile,
            target_id: NULL_ID,
            target_loc: [0.0, 0.0],
        };
        init.states.insert(id, unit);
        teams.insert(id, team);
    }
    (teams, init)
}

fn read_map(map: Vec<Vec<Vec2>>) -> path::Map {
    let mut result = path::Map::new();
    for poly in map {
        for i in 1..poly.len()-1 {
            result.push([poly[0], poly[i], poly[i+1]]);
        }
    }
    result
}
