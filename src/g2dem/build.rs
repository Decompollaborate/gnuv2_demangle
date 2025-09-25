/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");
}
