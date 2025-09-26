/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use gloo::storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use web_sys::HtmlSelectElement;
use yew::events::Event;
use yew::html::Scope;
use yew::{html, Component, Html, TargetCast};

mod demangling_style;
mod theme;

pub use demangling_style::DemanglingStyle;
pub use theme::Theme;

pub trait Storagable
where
    Self: Serialize + for<'de> Deserialize<'de>,
{
    fn storage_key() -> &'static str;

    fn load_storage<F>(default: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        LocalStorage::get(Self::storage_key()).unwrap_or_else(|_| default())
    }

    fn save_storage(self) {
        LocalStorage::set(Self::storage_key(), self)
            .expect("Failed to save key into LocalStorage.");
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum LabelPosition {
    #[default]
    Upper,
    Left,
}

pub trait DropdownEnum
where
    Self: Sized + PartialEq + 'static,
{
    fn from_id(id: &str) -> Self;
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn array() -> &'static [Self];
    fn label_text() -> &'static str;
    fn dropdown_id() -> &'static str;
    fn tooltip_text() -> Option<&'static str>;

    fn gen_dropdown<F, M, S>(
        &self,
        link: &Scope<S>,
        label_position: LabelPosition,
        msgfier: F,
    ) -> Html
    where
        F: Fn(Self) -> M + 'static,
        S: Component<Message = M>,
    {
        let label_text = Self::label_text();
        let dropdown_id = Self::dropdown_id();
        let onchange = link.batch_callback(move |e: Event| {
            let select: HtmlSelectElement = e.target_unchecked_into();
            Some(msgfier(Self::from_id(&select.value())))
        });

        // Wrap the label if there's tooltip text available.
        let label_text = if let Some(tooltip_text) = Self::tooltip_text() {
            html! {
              <span class="tooltip">
                { label_text }
                <span class="tooltiptext">{ tooltip_text }</span>
              </span>
            }
        } else {
            html! {
              <> {label_text} </>
            }
        };

        let elements: Vec<Html> = Self::array()
            .iter()
            .map(|x| {
                let selected = x == self;

                html! {
                    <option value={x.id()} {selected}> { {x.name()} } </option>
                }
            })
            .collect();

        let dropdown = html! {
          <select class="settings-dropdown" id={dropdown_id} {onchange}>
            { elements }
          </select>
        };

        match label_position {
            LabelPosition::Upper => html! {
              <label for={dropdown_id}> { label_text }
                { dropdown }
              </label>
            },
            LabelPosition::Left => html! {
              <>
                <label for={dropdown_id}> { label_text }</label>
                { dropdown }
              </>
            },
        }
    }
}
