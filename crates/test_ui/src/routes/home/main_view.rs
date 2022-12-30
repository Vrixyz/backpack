use yew::prelude::*;

use crate::hooks::use_user_context;

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct Props {}

/// Main content with tabs of article list for home page
#[function_component(MainView)]
pub fn main_view(_props: &Props) -> Html {
    let user_ctx = use_user_context();

    html! {
        <div class="col-md-9 col-xs-12">
            <div class="feed-toggle">
                <ul class="nav nav-pills outline-active">
                    {
                        if user_ctx.is_authenticated() {
                            html! {
                                <p>{ "User is authenticated" }</p>
                            }
                        } else {
                            html! {
                                <p>{ "User is not authenticated" }</p>
                            }
                        }
                    }
                </ul>
            </div>

        </div>
    }
}
