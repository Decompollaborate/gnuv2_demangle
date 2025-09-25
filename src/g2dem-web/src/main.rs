/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use yew::{html, Component, Context, Html};


pub enum Msg {
    InputData(String),
}

pub struct App {
    user_input: String,
    // state: PersistentState,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            user_input: "".to_string(),
            // state: PersistentState::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::InputData(input) => {
                self.user_input = input;
            },
        }

        // self.state.save();
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        /*
        let header = self.view_header(ctx);
        let main = self.view_main(ctx);
        let footer = self.view_footer(ctx);

        let root_class = format!("{} view-root", self.state.theme.id());

        use_effect(move || {
            // asm syntax highlight.
            // TODO: breaks the site due to lack of auto update
            // highlightAll();
        });

        html! {
          <div class={root_class}>
            { header }
            { main }
            { footer }
          </div>
        }
        */
        html! {
          <div>
            {"hi"}
          </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<App>::new().render();
}
