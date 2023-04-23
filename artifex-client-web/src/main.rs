//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use yew::prelude::*;

use artifex_client_web::{
    components::{Inspection, Settings},
    contexts::ServerProvider,
};

#[function_component]
fn App() -> Html {
    html! {
        <div>
          <h1>{ "Artifex Server Management" }</h1>
          <div class="server-management">
            <ServerProvider>
              <Settings />
              <Inspection />
            </ServerProvider>
          </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
