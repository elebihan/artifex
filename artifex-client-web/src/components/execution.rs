//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::contexts::ServerContext;
use crate::rpc::{create_client, ExecuteRequest};

pub enum ExecutionState {
    Failure(String),
    Idle,
    Ongoing,
    Success(String),
}

pub enum Msg {
    CommandChanged(String),
    Execute,
    SetExecutionState(ExecutionState),
    ServerUpdated(ServerContext),
}

pub struct Execution {
    server: ServerContext,
    state: ExecutionState,
    _listener: ContextHandle<ServerContext>,
    command: String,
}

impl Component for Execution {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (server, listener) = ctx
            .link()
            .context(ctx.link().callback(Msg::ServerUpdated))
            .expect("No server provided");
        Self {
            server,
            state: ExecutionState::Idle,
            _listener: listener,
            command: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::CommandChanged(command) => {
                self.command = command;
                true
            }
            Msg::Execute => {
                let mut client = create_client(&self.server.url);
                let command = self.command.clone();
                ctx.link().send_future(async move {
                    let state = match client.execute(ExecuteRequest { command }).await {
                        Ok(reply) => {
                            let reply = reply.into_inner();
                            ExecutionState::Success(reply.stdout)
                        }
                        Err(e) => ExecutionState::Failure(e.to_string()),
                    };
                    Msg::SetExecutionState(state)
                });
                ctx.link()
                    .send_message(Msg::SetExecutionState(ExecutionState::Ongoing));
                false
            }
            Msg::SetExecutionState(state) => {
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
        let oninput = ctx.link().callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Msg::CommandChanged(input.value())
        });
        let onclick = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Execute
        });
        let output = match &self.state {
            ExecutionState::Failure(text) => format!("Execution failed: {}", text),
            ExecutionState::Idle => "".to_string(),
            ExecutionState::Ongoing => "Execution in progress..".to_string(),
            ExecutionState::Success(text) => text.clone(),
        };
        html! {
            <div class="server-operations">
              <form>
                <label for="command">{"Execute command:"}</label>
                <input id="command" type="text" { oninput }/>
                <button id="execute" { onclick }>{"Execute"}</button>
                </form>
              <textarea rows=10 cols=80 readonly=true value={ output } />
            </div>
        }
    }
}
