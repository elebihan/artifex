//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::contexts::ServerContext;

#[function_component]
pub fn Settings() -> Html {
    let server = use_context::<ServerContext>().expect("Failed to get server context");
    let url = server.url.clone();
    let oninput = Callback::from(move |e: InputEvent| {
        let input: HtmlInputElement = e.target_unchecked_into();
        server.dispatch(input.value())
    });
    html! {
        <div class="server-information">
          <form>
            <label for="server-url">{"Server URL:"}</label>
            <input id="server-url" type="text" { oninput }
                   value={ url } />
          </form>
       </div>
    }
}
