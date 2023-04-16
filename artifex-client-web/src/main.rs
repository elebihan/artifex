//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use artifex_client_web::artifex_rpc::{artifex_client::ArtifexClient, InspectRequest};
use tonic_web_wasm_client::Client;
use web_sys::HtmlInputElement;
use yew::prelude::*;

const ARTIFEX_SERVER_URL: &'static str = "http://[::1]:50051";

fn create_client(url: &str) -> ArtifexClient<Client> {
    let client = Client::new(url.to_string());
    ArtifexClient::new(client)
}

enum InspectionState {
    Failure(String),
    Idle,
    Ongoing,
    Success(String),
}

enum Msg {
    UrlChanged(String),
    Inspect,
    SetInspectionState(InspectionState),
}

struct App {
    server_url: String,
    inspection: InspectionState,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            server_url: ARTIFEX_SERVER_URL.to_string(),
            inspection: InspectionState::Idle,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UrlChanged(address) => {
                self.server_url = address;
                true
            }
            Msg::Inspect => {
                let mut client = create_client(&self.server_url);
                ctx.link().send_future(async move {
                    let state = match client.inspect(InspectRequest {}).await {
                        Ok(reply) => {
                            let reply = reply.into_inner();
                            InspectionState::Success(format!(
                                "Kernel version: {}",
                                reply.kernel_version
                            ))
                        }
                        Err(e) => InspectionState::Failure(e.to_string()),
                    };
                    Msg::SetInspectionState(state)
                });
                ctx.link()
                    .send_message(Msg::SetInspectionState(InspectionState::Ongoing));
                false
            }
            Msg::SetInspectionState(state) => {
                self.inspection = state;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Msg::UrlChanged(input.value())
        });
        let onclick = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Inspect
        });
        let output = match &self.inspection {
            InspectionState::Failure(text) => format!("Inspection failed: {}", text),
            InspectionState::Idle => "".to_string(),
            InspectionState::Ongoing => "Inspection in progress..".to_string(),
            InspectionState::Success(text) => text.clone(),
        };
        html! {
        <div>
            <h1>{ "Artifex Server Management" }</h1>
            <div class="server-management">
                <div class="server-information">
                    <form>
                    <label for="server-url">{"Server URL:"}</label>
                    <input id="server-url" type="text" { oninput }
                           placeholder={ ARTIFEX_SERVER_URL }
                           value={ self.server_url.clone()} />
                    </form>
                </div>
                <div class="server-operations">
                    <form>
                    <label for="inspect">{"Inspect machine:"}</label>
                    <button id="inspect" { onclick }>{"Inspect"}</button>
                    </form>
                    <br />
                    <textarea rows=10 cols=80 readonly=true
                              value={ output } />
                </div>
            </div>
        </div> }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
