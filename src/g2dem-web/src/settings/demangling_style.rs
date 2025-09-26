/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use serde::{Deserialize, Serialize};

use crate::settings::{DropdownEnum, Storagable};

const KEY: &str = "decompollaborate.disasmdis-web.state.endian";

#[derive(
    Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub enum DemanglingStyle {
    #[default]
    G2dem,
    Cfilt,
}

impl Storagable for DemanglingStyle {
    fn storage_key() -> &'static str {
        KEY
    }
}

impl DropdownEnum for DemanglingStyle {
    fn from_id(id: &str) -> Self {
        match id {
            "g2dem" => Self::G2dem,
            "c++filt" => Self::Cfilt,
            _ => Self::default(),
        }
    }

    fn id(&self) -> &'static str {
        match self {
            Self::G2dem => "g2dem",
            Self::Cfilt => "c++filt",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::G2dem => "g2dem",
            Self::Cfilt => "c++filt",
        }
    }

    fn array() -> &'static [Self] {
        &ARR
    }

    fn label_text() -> &'static str {
        "Demangling style:"
    }
    fn dropdown_id() -> &'static str {
        "demangling_style"
    }

    fn tooltip_text() -> Option<&'static str> {
        Some("g2dem provides an slightly improved experience over the c++filt style, while the latter tries to mimic the original c++filt behavior.")
    }
}

static ARR: [DemanglingStyle; 2] = [DemanglingStyle::G2dem, DemanglingStyle::Cfilt];
