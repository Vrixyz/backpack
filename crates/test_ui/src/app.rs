use yew::prelude::*;
use yew_router::prelude::*;

use crate::prelude::nav::Nav;

// Use `std::alloc` as the global allocator.
#[global_allocator]
static ALLOC: std::alloc::System = std::alloc::System;

#[function_component(App)]
pub fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));

    html! {
        <BrowserRouter>
            <div class="flex flex-col h-screen">
                <Nav/>
            </div>
        </BrowserRouter>
    }
}
