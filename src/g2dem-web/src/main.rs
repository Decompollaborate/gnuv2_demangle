/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use web_sys::HtmlInputElement;
use yew::events::InputEvent;
use yew::html::Scope;
use yew::{html, Component, Context, Html, TargetCast};

use gnuv2_demangle::{demangle, DemangleConfig};

mod persistent_state;
mod settings;

use crate::persistent_state::PersistentState;
use crate::settings::*;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub enum Msg {
    InputData(String),
    ChangeTheme(Theme),
}

pub struct App {
    user_input: String,
    state: PersistentState,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            // TODO: have a list of mangled names and choose one randomly each time
            user_input: "test__Fv".to_string(),
            state: PersistentState::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::InputData(input) => {
                self.user_input = input;
            }
            Msg::ChangeTheme(theme) => {
                self.state.theme = theme;
            }
        }

        self.state.save();
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let header = self.view_header(ctx);
        let main = self.view_main(ctx);
        let footer = self.view_footer(ctx);

        let root_class = format!("{} view-root", self.state.theme.id());

        /*
        use_effect(move || {
            // asm syntax highlight.
            // TODO: breaks the site due to lack of auto update
            // highlightAll();
        });
        */

        html! {
          <div class={root_class}>
            { header }
            { main }
            { footer }
          </div>
        }
    }
}

impl App {
    fn view_header(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let label_position = LabelPosition::Left;

        let dropdown_theme_selector =
            self.state
                .theme
                .gen_dropdown(link, label_position, Msg::ChangeTheme);

        html! {
          <header>
            <h1> { "🧩 g2dem-web" } <h6> { built_info::PKG_VERSION } </h6> </h1>

            <div class="theme-selector">
              { dropdown_theme_selector }
            </div>
          </header>
        }
    }

    fn view_main(&self, ctx: &Context<Self>) -> Html {
        html! {
          <main>
            <section class="editor">
              { self.view_input(ctx.link()) }
              { self.view_output_box() }
            </section>
          </main>
        }
    }

    fn view_footer(&self, _ctx: &Context<Self>) -> Html {
        let git_info = if let Some(info) = built_info::GIT_COMMIT_HASH_SHORT {
            format!("Git hash: {info}")
        } else {
            String::new()
        };

        html! {
          <footer>
            <p> { "© 2025 Decompollaborate" } </p>
            <p> { "Powered by " } <a target="_blank" href={ built_info::PKG_REPOSITORY }>{ "gnuv2_demangle" }</a> </p>
            <p> { git_info } </p>
          </footer>
        }
    }
}

impl App {
    fn view_input(&self, link: &Scope<Self>) -> Html {
        let oninput = link.batch_callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Some(Msg::InputData(input.value()))
        });
        let placeholder = "Enter mangled symbols...";
        let value = self.user_input.clone();

        html! {
          <div class="input-box">
            <h2 for="bytes-input"> { "Input" } </h2>
            <textarea
              id="bytes-input"
              rows="8"
              cols="80"
              {placeholder}
              {oninput}
              {value}
            />
          </div>
        }
    }

    fn view_output_box(&self) -> Html {
        let result = self.demangle_input();
        let label = "Demangled output";

        html! {
          <div class="output-box">
            <h2> { label } </h2>
            <div class="scrollable-container">
              <pre><code /*class="language-mipsasm"*/>
                <table> { result } </table>
              </code></pre>
            </div>
          </div>
        }
    }

    fn demangle_input(&self) -> Vec<Html> {
        let mut result = Vec::new();
        let config = DemangleConfig::new_no_cfilt_mimics();

        for sym in self.user_input.lines() {
            let row = match demangle(sym.trim(), &config) {
                Ok(demangled) => html! {
                  <tr>
                    <td class="cod"> { demangled } </td>
                  </tr>
                },
                Err(_) => html! {
                  <tr>
                    <td class="cod"> { sym } </td>
                  </tr>
                },
            };
            result.push(row);
        }

        result
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<App>::new().render();
}
