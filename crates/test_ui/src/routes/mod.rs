use yew::prelude::*;
use yew_router::prelude::*;

/// App routes
#[derive(Routable, PartialEq, Eq, Clone, Debug)]
pub enum AppRoute {
    #[at("/login")]
    Login,

    #[at("/register")]
    Register,

    #[at("/")]
    Home,

    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(route: AppRoute) -> Html {
    match route {
        AppRoute::Login => todo!(),
        AppRoute::Register => todo!(),
        AppRoute::Home => todo!(),
        AppRoute::NotFound => html! { "Page not found" },
    }
}
