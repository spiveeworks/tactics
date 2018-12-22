extern crate piston_app;
extern crate piston_window;
extern crate vecmath;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate bincode;
extern crate toml;


mod model;
mod path;
mod server;
mod client;
mod server_app;
mod client_app;

pub mod prelude {
    use std::collections::HashMap;

    pub type EID = u32;
    pub type TID = u32;

    pub const NULL_ID: EID = EID::max_value();

    pub type Vec2 = ::vecmath::Vector2<f64>;
    pub use vecmath::{vec2_scale, vec2_add, vec2_sub};

    // should use NotNaN crate
    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub struct Time(pub f64);

    use std::cmp;
    impl PartialEq for Time {
        fn eq(self: &Self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    impl Eq for Time {}

    impl PartialOrd for Time {
        fn partial_cmp(self: &Self, other: &Self) -> Option<cmp::Ordering> {
            PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }

    impl Ord for Time {
        fn cmp(self: &Self, other: &Self) -> cmp::Ordering {
            cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                .expect("Got NaN as time...")
        }
    }

    pub fn empty_map<T, U: Default>(base: &HashMap<EID, T>) -> HashMap<EID, U> {
        base.iter()
            .map(|(&id, _)| (id, Default::default()))
            .collect()
    }
}

fn readln() -> String {
    let mut buffer = String::new();
    let stdin = ::std::io::stdin();

    stdin.read_line(&mut buffer).expect("Stdin failed");
    buffer.split_whitespace().next().expect("No ip entered").to_string()
}

fn main() {
    let mut args = std::env::args();
    let _this_app = args.next();
    let fst = args.next();
    let ip = readln();
    if fst == Some("-s".to_string()) {
        let path = args.next().unwrap_or("map".to_string());
        let app = server_app::ServerApp::new(&ip, path);
        app.run();
    } else {
        println!("Enter ip to connect to: ");
        let app = client_app::ClientApp::new(&ip);
        piston_app::run_until_escape(app);
    }
}
