mod banner;
mod main_view;

use main_view::MainView;
use yew::prelude::*;

/// Home page with an article list and a tag list.
#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <div class="home-page">
            <div class="container page">
                <div class="row">
                    <MainView />
                </div>
            </div>
        </div>
    }
}
