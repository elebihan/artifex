//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use yew::prelude::*;

use super::tab::Tab;

#[derive(Clone, PartialEq, Properties)]
pub struct TabListProps {
    #[prop_or_default]
    pub children: ChildrenWithProps<Tab>,
}

pub struct TabList {
    active_page: usize,
}

pub enum Msg {
    SwitchPage(usize),
}

impl Component for TabList {
    type Properties = TabListProps;
    type Message = Msg;

    fn create(_ctx: &Context<Self>) -> Self {
        TabList { active_page: 0 }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SwitchPage(index) => {
                self.active_page = index;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tabs = ctx.props().children.iter().enumerate().map(|(i, child)| {
            let is_active = i == self.active_page;
            let onclick = ctx.link().callback(move |_| Msg::SwitchPage(i));
            let title = child.props.title.clone();

            html! {
                <button role="tab" aria-selected={ if is_active { "true" } else { "false" } } { onclick }
                         aria-controls={ format!("tabpanel-{}", i)} >
                <span class="focus">{ title }</span>
               </button>
            }
        });
        let tab_panels = ctx.props().children.iter().enumerate().map(|(i, child)| {
            let is_active = i == self.active_page;
            let tab_panel_class = if is_active { "" } else { "hidden" };
            html! {
                <div id={ format!("tabpanel-{}", i) } role="tabpanel" class={tab_panel_class}>
                    {child.clone()}
                </div>
            }
        });

        html! {
            <div role="tablist">
                {for tabs}
                {for tab_panels}
            </div>
        }
    }
}
