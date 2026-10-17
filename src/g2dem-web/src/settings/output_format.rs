/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use serde::{Deserialize, Serialize};

use crate::settings::{DropdownEnum, Storagable};

const KEY: &str = "decompollaborate.g2dem-web.state.output_format";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum OutputFormat {
    #[default]
    Cpp,
    Tree,
}

impl Storagable for OutputFormat {
    fn storage_key() -> &'static str {
        KEY
    }
}

impl DropdownEnum for OutputFormat {
    fn from_id(id: &str) -> Self {
        match id {
            "c++" => Self::Cpp,
            "tree" => Self::Tree,
            _ => Self::default(),
        }
    }

    fn id(&self) -> &'static str {
        match self {
            Self::Cpp => "c++",
            Self::Tree => "tree",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Cpp => "C++",
            Self::Tree => "Tree",
        }
    }

    fn array() -> &'static [Self] {
        &ARR
    }

    fn label_text() -> &'static str {
        "Output format:"
    }
    fn dropdown_id() -> &'static str {
        "output_format"
    }

    fn tooltip_text() -> Option<&'static str> {
        Some("Choose how to visualize the demangled symbol, either as a C++ symbol or using a tree representation")
    }
}

static ARR: [OutputFormat; 2] = [OutputFormat::Cpp, OutputFormat::Tree];
