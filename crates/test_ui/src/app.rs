use yew::prelude::*;
use yew_router::prelude::*;

use crate::prelude::{
    footer::Footer, nav::Nav, switch, user_context_provider::UserContextProvider, AppRoute,
};

// Use `std::alloc` as the global allocator.
#[global_allocator]
static ALLOC: std::alloc::System = std::alloc::System;

#[function_component(App)]
pub fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));

    html! {
        <BrowserRouter>
            <UserContextProvider>
                <div class="flex flex-col h-screen">
                    <Nav/>
                    <Switch<AppRoute> render={switch} />
                    <Footer />
                </div>
            </UserContextProvider>
        </BrowserRouter>
    }
}
