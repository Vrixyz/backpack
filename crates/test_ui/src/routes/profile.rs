use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_router::prelude::*;

use crate::{hooks::use_user_context, routes::AppRoute, services::profiles::*};

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct Props {
    pub username: String,
    pub tab: ProfileTab,
}

#[derive(Clone, PartialEq, Eq)]
pub enum ProfileTab {
    ByAuthor,
    FavoritedBy,
}

/// Profile for an author
#[function_component(Profile)]
pub fn profile(props: &Props) -> Html {
    let profile_info = {
        let username = props.username.clone();
        use_async(async move { get(username).await })
    };
    let user_follow = {
        let profile_info = profile_info.clone();
        let username = props.username.clone();
        use_async(async move {
            if let Some(profile) = &profile_info.data {
                if profile.profile.following {
                    return unfollow(username).await;
                }
            }
            follow(username).await
        })
    };
    let user_ctx = use_user_context();
    let is_current_user = user_ctx.is_authenticated() && user_ctx.username == props.username;

    {
        let profile_info = profile_info.clone();
        use_effect_with_deps(
            move |_| {
                profile_info.run();
                || ()
            },
            props.username.clone(),
        );
    }

    {
        let profile_info = profile_info.clone();
        use_effect_with_deps(
            move |user_follow| {
                if let Some(profile) = &user_follow.data {
                    profile_info.update(profile.clone());
                }
                || ()
            },
            user_follow.clone(),
        );
    }

    let onclick = {
        Callback::from(move |_| {
            user_follow.run();
        })
    };

    profile_info.data.as_ref().map_or_else(|| html! {}, |profile| {
        let profile = &profile.profile;
        let classes_tab = if props.tab == ProfileTab::ByAuthor {
            ("nav-link active", "nav-link")
        } else {
            ("nav-link", "nav-link active")
        };

        let classes_follow = if profile.following {
            "btn btn-sm action-btn btn-secondary"
        } else {
            "btn btn-sm action-btn btn-outline-secondary"
        };

        let text = if profile.following { "Unfollow" } else { "Follow" };

        html! {
            <div class="profile-page">
                <div class="user-info">
                    <div class="container">
                        <div class="row">
                            <div class="col-xs-12 col-md-10 offset-md-1">
                                <img src={ profile.image.clone() } class="user-img" alt={ profile.username.clone() } />
                                <h4>{ &profile.username }</h4>
                                <p>
                                    {
                                        profile.bio.as_ref().map_or_else(|| html! { }, |bio| html! { bio })}
                                </p>
                                {
                                    if is_current_user {
                                        html! {
                                            <Link<AppRoute>
                                                to={AppRoute::Settings}
                                                classes="btn btn-sm btn-outline-secondary action-btn">
                                                { "Edit Profile Settings" }
                                            </Link<AppRoute>>
                                        }
                                    } else {
                                        html! {
                                            <button
                                                class={classes_follow}
                                                {onclick} >
                                                { text }
                                            </button>
                                        }
                                }}
                            </div>
                        </div>
                    </div>
                </div>

                <div class="container">
                    <div class="row">
                        <div class="col-xs-12 col-md-10 offset-md-1">
                            <div class="articles-toggle">
                                <ul class="nav nav-pills outline-active">
                                    // TODO: use yew-router Link
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    })
}
