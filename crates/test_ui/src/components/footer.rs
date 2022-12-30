use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::AppRoute;

#[function_component(Footer)]
pub fn footer() -> Html {
    html! {
        <footer class="p-4 bg-white rounded-lg shadow md:flex md:items-center md:justify-between md:p-6 dark:bg-gray-800">
            <Link<AppRoute> to={AppRoute::Home} classes="logo-font">{ "BackPack" }</Link<AppRoute>>

            <span class="text-sm text-gray-500 sm:text-center dark:text-gray-400">
                { "© 2023 " }
                <a href="https://github.com/Vrixyz/backpack" class="hover:underline">{"Backpack"}</a>
                { ". Code licensed under MIT." }
            </span>

            <ul class="flex flex-wrap items-center mt-3 text-sm text-gray-500 dark:text-gray-400 sm:mt-0">
                <li>
                    <a href="#" class="mr-4 hover:underline md:mr-6 ">{"About"}</a>
                </li>
                <li>
                    <a href="#" class="mr-4 hover:underline md:mr-6">{"Privacy Policy"}</a>
                </li>
                <li>
                    <a href="#" class="mr-4 hover:underline md:mr-6">{"Licensing"}</a>
                </li>
                <li>
                    <a href="#" class="hover:underline">{"Contact"}</a>
                </li>
            </ul>
        </footer>
    }
}
