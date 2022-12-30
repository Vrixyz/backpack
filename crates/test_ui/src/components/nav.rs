use yew::prelude::*;

use crate::{hooks::use_user_context, prelude::UserInfo};

#[function_component(Nav)]
pub fn nav() -> Html {
    let user_ctx = use_user_context();

    let link_classes = "block px-4 py-2 hover:bg-black hover:text-white rounded border-black border";

    let links = [
        ("Yew", "https://yew.rs/"),
        ("Tailwind", "https://tailwindcss.com"),
        ("Trunk", "https://github.com/thedodd/trunk"),
    ];

    html! {
        <nav class="bg-blue-400 h-16 px-8 py-2" role="navigation" aria-label="main navigation">
            <div class="container flex mx-auto gap-6 items-center h-full">
                <h1 class="font-bold text-2xl text-white">{"Trunk | Yew | Tailwind"}</h1>

                // <div class="flex-1"></div>
                // {for links.iter().map(|(label, href)| html! {
                //     <a class={link_classes} href={*href}>{label}</a>
                // })}

                {
                    if user_ctx.is_authenticated() {
                        logged_in_view((*user_ctx).clone())
                    } else {
                        logged_out_view()
                    }
                }
            </div>
        </nav>
    }
}

fn logged_out_view() -> Html {
    html! {
        <ul class="nav navbar-nav pull-xs-right">
            // <li class="nav-item">
            //     <Link<AppRoute> to={AppRoute::Home} classes="nav-link">
            //         { "Home" }
            //     </Link<AppRoute>>
            // </li>
            // <li class="nav-item">
            //     <Link<AppRoute> to={AppRoute::Login} classes="nav-link">
            //         { "Sign in" }
            //     </Link<AppRoute>>
            // </li>
            // <li class="nav-item">
            //     <Link<AppRoute> to={AppRoute::Register} classes="nav-link">
            //         { "Sign up" }
            //     </Link<AppRoute>>
            // </li>
        </ul>
    }
}

fn logged_in_view(user_info: UserInfo) -> Html {
    html! {
        <ul class="nav navbar-nav pull-xs-right">
            // <li class="nav-item">
            //     <Link<AppRoute> to={AppRoute::Home} classes="nav-link">
            //         { "Home" }
            //     </Link<AppRoute>>
            // </li>
            // <li class="nav-item">
            //     <Link<AppRoute> to={AppRoute::EditorCreate} classes="nav-link">
            //         { "New Post" }
            //     </Link<AppRoute>>
            // </li>
            // <li class="nav-item">
            //     <Link<AppRoute> to={AppRoute::Settings} classes="nav-link">
            //         { "Settings" }
            //     </Link<AppRoute>>
            // </li>
            // <li class="nav-item">
            //     <Link<AppRoute> to={AppRoute::Profile { username: user_info.username.clone() }}  classes="nav-link">
            //         { &user_info.username }
            //     </Link<AppRoute>>
            // </li>
        </ul>
    }
}
