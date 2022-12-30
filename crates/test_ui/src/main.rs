#![warn(clippy::nursery, clippy::all)]
#![recursion_limit = "1024"]

mod app;
mod components;
mod error;
mod hooks;
mod routes;
mod services;
mod types;

pub mod prelude {
    pub use crate::{components::*, error::*, hooks::*, routes::*, services::*, types::*};
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    yew::Renderer::<app::App>::new().render();
}
