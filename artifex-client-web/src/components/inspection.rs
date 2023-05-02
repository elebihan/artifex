//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use yew::prelude::*;

use crate::contexts::ServerContext;
use crate::rpc::{create_client, InspectRequest};

pub enum InspectionState {
    Failure(String),
    Idle,
    Ongoing,
    Success(String),
}

pub enum Msg {
    Inspect,
    SetInspectionState(InspectionState),
    ServerUpdated(ServerContext),
}

pub struct Inspection {
    server: ServerContext,
    state: InspectionState,
    _listener: ContextHandle<ServerContext>,
}

impl Component for Inspection {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (server, listener) = ctx
            .link()
            .context(ctx.link().callback(Msg::ServerUpdated))
            .expect("No server provided");
        Self {
            server,
            state: InspectionState::Idle,
            _listener: listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Inspect => {
                let mut client = create_client(&self.server.url);
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
                self.state = state;
                true
            }
            Msg::ServerUpdated(server) => {
                self.server = server;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Inspect
        });
        let output = match &self.state {
            InspectionState::Failure(text) => format!("Inspection failed: {}", text),
            InspectionState::Idle => "".to_string(),
            InspectionState::Ongoing => "Inspection in progress..".to_string(),
            InspectionState::Success(text) => text.clone(),
        };
        html! {
            <div class="server-operations">
              <form>
                <label for="inspect">{"Inspect machine:"}</label>
                <button id="inspect" { onclick }>{"Inspect"}</button>
                </form>
              <br />
              <textarea rows=10 cols=80 readonly=true value={ output } />
            </div>
        }
    }
}
