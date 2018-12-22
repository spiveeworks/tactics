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

pub fn gen_map() {
    let units = vec![
        Unit { team: 0, pos: (5.0, 30.0), weapon: model::Weapon::Gun },
        Unit { team: 0, pos: (55.0, 30.0), weapon: model::Weapon::Gun },
        Unit { team: 1, pos: (30.0, 5.0), weapon: model::Weapon::Gun },
        Unit { team: 1, pos: (30.0, 55.0), weapon: model::Weapon::Gun },
    ];

    let mut map = Vec::new();

    let polys = [
        // centre block
        vec![[29.0,29.0],[29.0,30.0],[30.0,30.0],[30.0,29.0]],
        // centre walls
        vec![[20.0,20.0],[27.0,20.0],[27.0,21.0],
             [21.0,21.0],[21.0,27.0],[20.0,27.0]],
        // outer diags
        vec![[2.0, 2.0],[2.0,3.0],[15.0,16.0],
             [16.0,16.0],[16.0,15.0],[3.0,2.0]],
        // inner diags
        vec![[10.0,20.0],[10.0,20.5],[17.5,28.0],
             [18.0,28.0],[18.0,27.5],[10.5,20.0]],
        vec![[20.0,10.0],[20.0,10.5],[27.5,18.0],
             [28.0,18.0],[28.0,17.5],[20.5,10.0]],
    ];
    let fns = [[0.0,1.0],[60.0,-1.0]];
    for poly in polys.iter() {
        for fx in fns.iter() {
            for fy in fns.iter() {
                let f = |p: [f64;2]|
                    [fx[0]+fx[1]*p[0],fy[0]+fy[1]*p[1]];
                let poly = poly.iter().cloned().map(f).collect();
                map.push(poly);
            }
        }
    }

    let config = Default::default();
    let toml = ::ron::ser::to_string_pretty(&Scenario{units, map}, config)
        .unwrap();
    let mut file = ::std::fs::File::create("demo").unwrap();
    use std::io::Write;
    file.write_all(toml.as_bytes()).unwrap();
}
