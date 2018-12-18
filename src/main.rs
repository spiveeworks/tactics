extern crate piston_app;
extern crate piston_window;
extern crate vecmath;


mod model;
mod path;
mod server;
mod client;
mod client_app;

pub mod prelude {
    use std::collections::HashMap;

    pub type EID = usize;

    pub const NULL_ID: EID = EID::max_value();

    pub type Vec2 = ::vecmath::Vector2<f64>;
    pub use vecmath::{vec2_scale, vec2_add};

    // should use NotNaN crate
    #[derive(Clone, Copy)]
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



fn main() {
    let app = client_app::ClientApp::new_demo();
    piston_app::run_until_escape(app);
}
