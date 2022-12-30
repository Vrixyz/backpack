use yew::prelude::*;
use yew_router::prelude::Link;

use crate::prelude::*;

#[function_component(Nav)]
pub fn nav() -> Html {
    let user_ctx = use_user_context();

    html! {
        // <nav class="bg-blue-400 h-16 px-8 py-2" role="navigation" aria-label="main navigation">
        //     <div class="container flex mx-auto gap-6 items-center h-full">
        //         <h1 class="font-bold text-2xl text-white">{"Trunk | Yew | Tailwind"}</h1>

        //         {
        //             if user_ctx.is_authenticated() {
        //                 logged_in_view((*user_ctx).clone())
        //             } else {
        //                 logged_out_view()
        //             }
        //         }
        //     </div>
        // </nav>

        <nav class="bg-white border-gray-200 px-2 sm:px-4 py-2.5 rounded dark:bg-gray-900">
            <div class="container flex flex-wrap items-center justify-between mx-auto">
                <a href="/" class="flex items-center">
                    <img src="https://flowbite.com/docs/images/logo.svg" class="h-6 mr-3 sm:h-9" alt="Flowbite Logo" />
                    <span class="self-center text-xl font-semibold whitespace-nowrap dark:text-white">{"Backpack"}</span>
                </a>

                <button data-collapse-toggle="navbar-default" type="button" class="inline-flex items-center p-2 ml-3 text-sm text-gray-500 rounded-lg md:hidden hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-200 dark:text-gray-400 dark:hover:bg-gray-700 dark:focus:ring-gray-600" aria-controls="navbar-default" aria-expanded="false">
                    <span class="sr-only">{"Open main menu"}</span>
                    <svg class="w-6 h-6" aria-hidden="true" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                        <path fill-rule="evenodd" d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clip-rule="evenodd"></path>
                    </svg>
                </button>

                <div class="hidden w-full md:block md:w-auto" id="navbar-default">
                    <ul class="flex flex-col p-4 mt-4 border border-gray-100 rounded-lg bg-gray-50 md:flex-row md:space-x-8 md:mt-0 md:text-sm md:font-medium md:border-0 md:bg-white dark:bg-gray-800 md:dark:bg-gray-900 dark:border-gray-700">
                    <li>
                        <a href="/" class="block py-2 pl-3 pr-4 text-white bg-blue-700 rounded md:bg-transparent md:text-blue-700 md:p-0 dark:text-white" aria-current="page">{"Home"}</a>
                    </li>
                    <li>
                        <a href="/about" class="block py-2 pl-3 pr-4 text-gray-700 rounded hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 dark:text-gray-400 md:dark:hover:text-white dark:hover:bg-gray-700 dark:hover:text-white md:dark:hover:bg-transparent">{"About"}</a>
                    </li>
                    </ul>
                </div>
            </div>
        </nav>
    }
}

fn logged_out_view() -> Html {
    html! {
        <ul class="nav navbar-nav pull-xs-right">
            <li class="nav-item">
                <Link<AppRoute> to={AppRoute::Home} classes="nav-link">
                    { "Home" }
                </Link<AppRoute>>
            </li>
            <li class="nav-item">
                <Link<AppRoute> to={AppRoute::Login} classes="nav-link">
                    { "Sign in" }
                </Link<AppRoute>>
            </li>
            <li class="nav-item">
                <Link<AppRoute> to={AppRoute::Register} classes="nav-link">
                    { "Sign up" }
                </Link<AppRoute>>
            </li>
        </ul>
    }
}

fn logged_in_view(user_info: UserInfo) -> Html {
    html! {
        <ul class="nav navbar-nav pull-xs-right">
            <li class="nav-item">
                <Link<AppRoute> to={AppRoute::Home} classes="nav-link">
                    { "Home" }
                </Link<AppRoute>>
            </li>
            <li class="nav-item">
                <Link<AppRoute> to={AppRoute::Profile { username: user_info.username.clone() }}  classes="nav-link">
                    { &user_info.username }
                </Link<AppRoute>>
            </li>
        </ul>
    }
}
