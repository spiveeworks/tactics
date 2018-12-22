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
}

impl ServerApp {
    pub fn new<I: net::ToSocketAddrs>(ip: I) -> Self {
        let listener = net::TcpListener::bind(ip)
            .expect("Failed to connect to server");
        let players = HashMap::new();
        let (teams, init) = Self::demo_snap();
        let map = Self::demo_map();
        let server = Server::new(init, map);
        ServerApp { listener, teams, players, server }
    }

    fn demo_snap() -> (HashMap<EID, TID>, model::Snapshot) {
        let mut teams = HashMap::new();
        let mut init = model::Snapshot {
            time: 0.0,
            states: HashMap::new(),
        };
        let unit = model::UnitState {
            id: 0,
            pos: [30.0, 30.0],
            vel: [0.0, 0.0],
            time: 0.0,

            weapon: model::Weapon::Gun,
            action: model::Action::Mobile,
            target_id: NULL_ID,
            target_loc: [0.0, 0.0],
        };
        let mut units = [unit;4];
        units[0].pos[0] = 5.0;
        units[1].pos[0] = 55.0;
        units[2].pos[1] = 5.0;
        units[3].pos[1] = 55.0;
        for i in 0..4 {
            teams.insert(i as EID, if i < 2 { 0 } else { 1 });
            units[i].id = i as EID;
            init.states.insert(i as EID, units[i]);
        }

        (teams, init)
    }

    fn demo_map() -> path::Map {
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

        map
    }

    pub fn run(mut self: Self) {
        for (_unit, team) in self.teams.clone() {
            if !self.players.contains_key(&team) {
                println!("Waiting for player {}", team);
                let player = self.listener.accept().unwrap().0;
                self.players.insert(team, player);
                self.introduce(team);
            }
        }
        loop {
        }
    }

    fn introduce(self: &mut Self, team: TID) {
        let player = &self.players[&team];
        ::bincode::serialize_into(player, &self.server.map)
            .expect("Failed to send map");
        ::bincode::serialize_into(player, &self.server.current)
            .expect("Failed to send unit state");
    }
}
