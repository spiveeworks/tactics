use std::collections::HashMap;
use std::net;

use prelude::*;

use model;
use save;
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
        save::gen_map();
        let (teams, init, map) = save::read_scenario(path);
        let server = Server::new(init, map);
        //let read_timeout = Some(time::Duration::from_millis(100));
        ServerApp { listener, teams, players, server, }
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
