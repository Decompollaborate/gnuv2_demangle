/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use log::info;

use crate::settings::*;

pub struct PersistentState {
    pub theme: Theme,
}

impl PersistentState {
    pub fn new() -> Self {
        Self {
            theme: Storagable::load_storage(Default::default),
        }
    }

    pub fn save(&self) {
        let Self { theme } = self;

        info!("Saving theme: {theme:?}");

        theme.save_storage();
    }
}
