extern crate piston_app;
extern crate piston_window;

use piston_window as window;

struct Client;

impl piston_app::App for Client {
    fn on_draw(
        self: &mut Self,
        centre: window::Context,
        graphics: &mut window::G2d,
        args: window::RenderArgs,
    ) {
    }

    fn on_update(
        self: &mut Self,
        args: window::UpdateArgs,
    ) {
    }
    fn on_input(
        self: &mut Self,
        args: window::ButtonArgs,
    ) {
    }
    fn on_mouse_move(
        self: &mut Self,
        mouse: [f64; 2],
    ) {
    }

    fn window_name() -> &'static str {
        "perfect information tactical planning (prototype)"
    }
    fn window_starting_size() -> [u32; 2] {
        [600, 600]
    }
}


fn main() {
    let client = Client;
    piston_app::run_until_escape(client);
}
