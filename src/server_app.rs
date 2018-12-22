use std::collections::HashMap;
use std::net;

use prelude::*;

pub struct ServerApp {
    listener: net::TcpListener,
    teams: HashMap<EID, TID>,
    players: HashMap<TID, net::TcpStream>,
}

impl ServerApp {
    pub fn new<I: net::ToSocketAddrs>(ip: I) -> Self {
        let listener = net::TcpListener::bind(ip)
            .expect("Failed to connect to server");
        let teams = HashMap::new();
        let players = HashMap::new();
        ServerApp { listener, teams, players }
    }

    pub fn run(self: Self) {
        loop{
        }
    }
}
