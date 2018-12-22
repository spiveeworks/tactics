use std::collections::HashMap;
use std::net;

use prelude::*;

use model;
use path;
use server::Server;

pub struct ServerApp {
    listener: net::TcpListener,
    teams: HashMap<EID, TID>,
    players: HashMap<TID, net::TcpStream>,
    server: Server,
    // read_timeout: Option<time::Duration>,
}

impl ServerApp {
    pub fn new<I: net::ToSocketAddrs>(ip: I, path: String) -> Self {
        let listener = net::TcpListener::bind(ip)
            .expect("Failed to connect to server");
        let players = HashMap::new();
        Self::gen_map();
        let (teams, init, map) = Self::read_scenario(path);
        let server = Server::new(init, map);
        //let read_timeout = Some(time::Duration::from_millis(100));
        ServerApp { listener, teams, players, server, }
    }

    fn read_scenario(path: String)
        -> (HashMap<EID, TID>, model::Snapshot, path::Map)
    {
        let mut file = ::std::fs::File::open(path)
            .expect("Couldn't open file");
        let mut stuff = String::new();
        use std::io::Read;
        file.read_to_string(&mut stuff)
            .expect("Couldn't read file");

        #[derive(Deserialize)]
        struct Unit {
            team: TID,
            pos: (f64, f64),
            weapon: model::Weapon,
        }
        let (units, map): (Vec<Unit>, path::Map) = ::toml::from_str(&stuff)
            .expect("Failed to read file");

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
        (teams, init, map)
    }
    fn gen_map() {
        #[derive(Serialize)]
        struct Unit {
            team: TID,
            pos: (f64, f64),
            weapon: model::Weapon,
        }
        let units = vec![
            Unit { team: 0, pos: (5.0, 30.0), weapon: model::Weapon::Gun },
            Unit { team: 0, pos: (55.0, 30.0), weapon: model::Weapon::Gun },
            Unit { team: 1, pos: (30.0, 5.0), weapon: model::Weapon::Gun },
            Unit { team: 1, pos: (30.0, 55.0), weapon: model::Weapon::Gun },
        ];

        let mut map = path::Map::new();

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
            let len = poly.len();
            for fx in fns.iter() {
                for fy in fns.iter() {
                    for i in 1..len {
                        let j = (i+1)%len;
                        let f = |p: [f64;2]|
                            [fx[0]+fx[1]*p[0],fy[0]+fy[1]*p[1]];
                        map.push([f(poly[0]), f(poly[i]), f(poly[j])]);
                    }
                }
            }
        }

        let toml = ::toml::to_string(&(units, map)).unwrap();
        let mut file = ::std::fs::File::create("demo").unwrap();
        use std::io::Write;
        file.write_all(toml.as_bytes()).unwrap();
    }

    pub fn run(mut self: Self) {
        for (_unit, team) in self.teams.clone() {
            if !self.players.contains_key(&team) {
                println!("Waiting for player {}", team);
                let mut player = self.listener.accept().unwrap().0;
                // player.set_read_timeout(self.read_timeout);
                ::bincode::serialize_into(&player, &self.server.map)
                    .expect("Failed to send map");
                ::bincode::serialize_into(&player, &self.server.current)
                    .expect("Failed to send unit state");
                self.players.insert(team, player);
            }
        }
        loop {
            let mut plans = HashMap::new();
            for (&team, player) in &self.players {
                let plan: HashMap<EID, model::UnitState> =
                    ::bincode::deserialize_from(player)
                    .expect(
                        &format!("Failed to get plan from player {}", team)
                    );
                plans.insert(team, plan);
            }
            let mut moves = Vec::with_capacity(self.teams.len());
            for (&unit, &team) in &self.teams {
                let plan = &plans[&team];
                if plan.contains_key(&unit) {
                    moves.push(plan[&unit]);
                }
            }
            let result = self.server.resolve(moves.into_iter())
                .expect("Player submitted invalid move");
            for (_, player) in &self.players {
                ::bincode::serialize_into(player, &result)
                    .expect("Failed to send server result");
            }
        }
    }
}
