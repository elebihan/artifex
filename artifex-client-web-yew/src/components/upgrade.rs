//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use futures_util::StreamExt;
use tonic::codec::Streaming;
use yew::prelude::*;

use crate::{
    contexts::ServerContext,
    rpc::{create_client, UpgradeReply, UpgradeRequest},
};

pub enum UpgradeState {
    Complete,
    Failure(String),
    Idle,
    Progressed(usize),
    Started,
}

pub enum Msg {
    ServerUpdated(ServerContext),
    Upgrade,
    UpgradeFailed(String),
    UpgradeProgressed(usize),
    UpgradeStarted(Streaming<UpgradeReply>),
}

pub struct Upgrade {
    server: ServerContext,
    state: UpgradeState,
    _listener: ContextHandle<ServerContext>,
}

impl Component for Upgrade {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (server, listener) = ctx
            .link()
            .context(ctx.link().callback(Msg::ServerUpdated))
            .expect("No server provided");
        Self {
            server,
            state: UpgradeState::Idle,
            _listener: listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ServerUpdated(server) => {
                self.server = server;
                true
            }
            Msg::Upgrade => {
                let mut client = create_client(&self.server.url);
                ctx.link().send_future(async move {
                    match client.upgrade(UpgradeRequest {}).await {
                        Ok(stream) => Msg::UpgradeStarted(stream.into_inner()),
                        Err(e) => Msg::UpgradeFailed(e.to_string()),
                    }
                });
                false
            }
            Msg::UpgradeFailed(message) => {
                self.state = UpgradeState::Failure(message);
                true
            }
            Msg::UpgradeProgressed(position) => {
                self.state = if position == 100 {
                    UpgradeState::Complete
                } else {
                    UpgradeState::Progressed(position)
                };
                true
            }
            Msg::UpgradeStarted(stream) => {
                ctx.link().send_stream(stream.map(|item| match item {
                    Ok(reply) => Msg::UpgradeProgressed(reply.position as usize),
                    Err(e) => Msg::UpgradeFailed(e.message().to_string()),
                }));
                self.state = UpgradeState::Started;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Upgrade
        });
        let (value, text) = match &self.state {
            UpgradeState::Complete => (100, "Success".to_string()),
            UpgradeState::Failure(message) => (0, format!("Error: {}", message)),
            UpgradeState::Idle => (0, "".to_string()),
            UpgradeState::Progressed(position) => (*position, format!("{}%", *position)),
            UpgradeState::Started => (0, "Started".to_string()),
        };
        html! {
            <div class="server-operations">
              <form>
                <label for="start">{"Perform upgrade"}</label>
                <button id="start" { onclick }>{"Start"}</button>
              </form>
              <progress id="progress-bar" role="progress" max="100" value={ value.to_string() } />
              <label for="progress-bar">{ text }</label>
            </div>
        }
    }
}
