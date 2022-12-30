//! User context provider.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use yew::prelude::*;
use yew_hooks::prelude::*;

use crate::{
    error::Error,
    services::{auth::*, get_token, set_token},
    types::UserInfo,
};

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Children,
}

/// User context provider.
#[function_component(UserContextProvider)]
pub fn user_context_provider(props: &Props) -> Html {
    let user_ctx = use_state(UserInfo::default);
    let current_user = use_async(async move { current().await });

    // {
    //     let current_user = current_user.clone();
    //     use_mount(move || {
    //         if get_token().is_some() {
    //             current_user.run();
    //         }
    //     });
    // }

    // {
    //     let user_ctx = user_ctx.clone();
    //     use_effect_with_deps(
    //         move |current_user| {
    //             if let Some(user_info) = &current_user.data {
    //                 user_ctx.set(user_info.user.clone());
    //             }

    //             if let Some(error) = &current_user.error {
    //                 match error {
    //                     Error::Unauthorized | Error::Forbidden => set_token(None),
    //                     _ => (),
    //                 }
    //             }
    //             || ()
    //         },
    //         current_user,
    //     )
    // }

    html! {
        <ContextProvider<UseStateHandle<UserInfo>> context={user_ctx}>
            { for props.children.iter() }
        </ContextProvider<UseStateHandle<UserInfo>>>
    }
}

/// `use_async` demo
#[function_component(UseAsync)]
pub fn async_demo() -> Html {
    let repo = use_state(|| "jetli/yew-hooks".to_string());
    // Demo #1, manually call `run` to load data.
    let state = {
        let repo = repo.clone();
        use_async(async move { fetch_repo((*repo).clone()).await })
    };

    let onclick = {
        let state = state.clone();
        Callback::from(move |_| {
            // You can manually trigger to run in callback or use_effect.
            state.run();
        })
    };

    let oninput = {
        let repo = repo.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            repo.set(input.value());
        })
    };

    // Demo #2, automatically load data when mount
    let _ = {
        let repo = repo.clone();
        use_async_with_options(
            async move { fetch_repo((*repo).clone()).await },
            // This will load data automatically when mount.
            UseAsyncOptions::enable_auto(),
        )
    };

    html! {
        <div class="app">
            <header class="app-header">
                <div>
                    <input placeholder="Repo" value={(*repo).clone()} {oninput}/>
                    <button {onclick} disabled={state.loading}>{ "Start to load repo" }</button>
                    <p>
                        {
                            if state.loading {
                                html! { "Loading, wait a sec..." }
                            } else {
                                html! {}
                            }
                        }
                    </p>
                    {
                        state.data.as_ref().map_or_else(|| html! {}, |repo| html! {
                            <>
                                <p>{ "Repo name: " }<b>{ &repo.name }</b></p>
                                <p>{ "Repo full name: " }<b>{ &repo.full_name }</b></p>
                                <p>{ "Repo description: " }<b>{ &repo.description }</b></p>

                                <p>{ "Owner name: " }<b>{ &repo.owner.login }</b></p>
                                <p>{ "Owner avatar: " }<b><br/><img alt="avatar" src={repo.owner.avatar_url.clone()} /></b></p>
                            </>
                            })
                    }
                    <p>
                        {
                            state.error.as_ref().map_or_else(|| html! {}, |error| match error {
                                Error::DeserializeError => html! { "DeserializeError" },
                                Error::RequestError => html! { "RequestError" },
                            })
                        }
                    </p>
                </div>
            </header>
        </div>
    }
}

async fn fetch_repo(repo: String) -> Result<Repo, Error2> {
    fetch::<Repo>(format!("https://api.github.com/repos/{}", repo)).await
}

/// You can use reqwest or other crates to fetch your api.
async fn fetch<T>(url: String) -> Result<T, Error2>
where T: DeserializeOwned {
    let response = reqwest::get(url).await;
    if let Ok(data) = response {
        (data.json::<T>().await).map_or(Err(Error2::DeserializeError), |repo| Ok(repo))
    } else {
        Err(Error2::RequestError)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct User {
    id: i32,
    login: String,
    avatar_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Repo {
    id: i32,
    name: String,
    full_name: String,
    description: String,
    owner: User,
}

// You can use thiserror to define your errors.
#[derive(Clone, Debug, PartialEq)]
enum Error2 {
    RequestError,
    DeserializeError,
    // etc.
}
