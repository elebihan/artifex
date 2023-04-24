//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TabProps {
    pub title: String,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Tab)]
pub fn tab(props: &TabProps) -> Html {
    html! {
        { for props.children.clone() }
    }
}
