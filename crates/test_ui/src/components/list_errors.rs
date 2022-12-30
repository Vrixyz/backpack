use yew::prelude::*;

use crate::error::Error;

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct Props {
    pub error: Option<Error>,
}

#[function_component(ListErrors)]
pub fn list_errors(props: &Props) -> Html {
    props.error.as_ref().map_or_else(
        || html! {},
        |error| {
            html! {
                <ul class="error-messages">
                    {
                        match error {
                            Error::UnprocessableEntity(error_info) => {
                                html! {
                                    <>
                                    {for error_info.errors.iter().map(|(key, value)| {
                                        html! {
                                            <li>
                                            { key }
                                            {for value.iter().map(|e| {
                                                html! {
                                                    <>{" "} {e}</>
                                                }
                                            })}
                                            </li>
                                        }
                                    })}
                                    </>
                                }
                            }
                            _ => {
                                html! {
                                    <li>{error}</li>
                                }
                            }

                        }
                    }
                </ul>
            }
        },
    )
}
