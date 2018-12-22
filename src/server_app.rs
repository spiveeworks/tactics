use std::collections::HashMap;
use std::net;

use prelude::*;

use model;
use save;
use server::Server;

pub struct ServerApp {
    listener: net::TcpListener,
}

pub struct ServerInstance {
    teams: HashMap<EID, TID>,
    players: HashMap<TID, net::TcpStream>,
    player_names: HashMap<TID, String>,
    server: Server,
    // read_timeout: Option<time::Duration>,
}

impl ServerApp {
    pub fn new<I: net::ToSocketAddrs>(ip: I) -> Self {
        let listener = net::TcpListener::bind(ip)
            .expect("Failed to connect to server");
        ServerApp { listener }
    }

    pub fn spawn_instance(self: &mut Self, path: &String) -> ServerInstance {
        save::gen_map();
        let mut instance = ServerInstance::new(path); 
        for (_unit, team) in instance.teams.clone() {
            if !instance.players.contains_key(&team) {
                println!("Waiting for player {}", team);
                let player = self.listener.accept().unwrap().0;
                instance.add_player(team, player);
            }
        }
        instance.send_roster();
        instance
    }
}

impl ServerInstance {
    fn new(path: &String) -> Self {
        let (teams, init, map) = save::read_scenario(path);
        let server = Server::new(init, map);
        //let read_timeout = Some(time::Duration::from_millis(100));
        ServerInstance {
            teams,
            server,

            players: HashMap::new(),
            player_names: HashMap::new(),
        }
    }

    fn add_player(self: &mut Self, team: TID, player: net::TcpStream) {
        //player.set_nonblocking(true).unwrap();
        let name = bincode::deserialize_from(&player)
            .expect("Failed to read player name");
        ::bincode::serialize_into(&player, &self.server.map)
            .expect("Failed to send map");
        ::bincode::serialize_into(&player, &self.server.current)
            .expect("Failed to send unit state");
        self.player_names.insert(team, name);
        self.players.insert(team, player);
    }

    fn send_roster(self: &Self) {
        let mut intro = "The following players have joined: \n".to_string();
        for (team, name) in &self.player_names {
            intro.push_str(&*format!(" {}: {}\n", team+1, name));
        }
        for (_, player) in &self.players {
            ::bincode::serialize_into(player, &intro)
                .expect("Failed to send player roster");
        }
        print!("{}", &intro);
    }

    fn recv_plans(self: &Self) -> HashMap<TID, HashMap<EID, model::UnitState>>
    {
        let mut plans = HashMap::new();
        for (&team, player) in &self.players {
            let plan: HashMap<EID, model::UnitState> =
                ::bincode::deserialize_from(player)
                .expect(
                    &format!("Failed to get plan from player {}", team)
                );
            plans.insert(team, plan);
        }
        plans
    }

    fn take_plans(
        self: &mut Self,
        plans: HashMap<TID, HashMap<EID, model::UnitState>>,
    ) -> model::Snapshot {
        let mut moves = Vec::with_capacity(self.teams.len());
        for (&unit, &team) in &self.teams {
            let plan = &plans[&team];
            if plan.contains_key(&unit) {
                moves.push(plan[&unit]);
            }
        }
        let result = self.server.resolve(moves.into_iter())
            .expect("Player submitted invalid move");
        result
    }

    fn send_results(self: &mut Self, result: model::Snapshot) {
        for (_, player) in &self.players {
            ::bincode::serialize_into(player, &result)
                .expect("Failed to send server result");
        }
    }

    pub fn run(mut self: Self) {
        loop {
            let plans = self.recv_plans();
            let result = self.take_plans(plans);
            self.send_results(result);
        }
    }

    pub fn run_async(self: Self) {
        ::std::thread::spawn(move || self.run());
    }
}
