use yew::prelude::*;
use yew_router::prelude::*;

pub mod home;
pub mod login;
pub mod profile;
pub mod register;
pub mod settings;

use home::Home;
use login::Login;
use profile::{Profile, ProfileTab};
use register::Register;
use settings::Settings;

/// App routes
#[derive(Routable, PartialEq, Eq, Clone, Debug)]
pub enum AppRoute {
    #[at("/login")]
    Login,

    #[at("/register")]
    Register,

    #[at("/")]
    Home,

    #[at("/:username")]
    Profile { username: String },

    #[at("/settings")]
    Settings,

    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(route: AppRoute) -> Html {
    match route {
        AppRoute::Login => html! {<Login />},
        AppRoute::Register => html! {<Register />},
        AppRoute::Home => html! {<Home />},
        AppRoute::Profile { username } => html! {
            <Profile username={username} tab={ProfileTab::ByAuthor} />
        },
        AppRoute::Settings => html! {<Settings />},
        AppRoute::NotFound => html! { "Page not found" },
    }
}
