//! Architectural locks for the private renderer/protocol/tree module seams.
//!
//! This is deliberately a small text-level guard, not a Rust parser. When adding a
//! renderer, protocol, or tree child module, add its filename to the module lists
//! below, then add only the intentional `super::` edge or crate-level import prefix
//! to the corresponding allowlist. Keep implementation exports `pub(super)`/`pub(crate)`;
//! the one public `start_commit_reader` method is an explicit preserved API exception.

use std::fs;
use std::path::{Path, PathBuf};

const RENDERER_CHILD_MODULES: &[&str] = &[
    "animation.rs",
    "commands.rs",
    "commit_reader.rs",
    "events.rs",
    "input.rs",
];
const PAINT_CHILD_MODULES: &[&str] = &[
    "accessibility.rs",
    "drag.rs",
    "image.rs",
    "overlay.rs",
    "style.rs",
    "text_input.rs",
    "virtual_list.rs",
];
const PROTOCOL_CHILD_MODULES: &[&str] = &[
    "wire/mod.rs",
    "wire/snapshot_patch.rs",
    "wire/node.rs",
    "wire/command.rs",
    "wire/event.rs",
];

const TREE_CHILD_MODULES: &[&str] = &["validation.rs"];

// These are the current intentional parent/sibling edges. A new edge must be
// reviewed and added here explicitly rather than becoming an accidental cycle.
const ALLOWED_RENDERER_SUPER_IMPORTS: &[&str] = &[
    "use super::ReactRoot",
    "use super::committed_child_index",
    "use super::events::",
    "use super::RenderedBounds",
    "use super::super::ReactRoot",
    "use super::super::committed_child_index",
    "use super::super::events::",
    "use super::accessibility::",
    "use super::style::",
    // The input module's focused unit tests import their parent module.
    "use super::*",
];
const ALLOWED_INTERNAL_VISIBILITY: &[&str] = &["pub(super) ", "pub(crate) "];
const ALLOWED_PUBLIC_ITEMS: &[(&str, &str)] =
    &[("renderer/commit_reader.rs", "pub fn start_commit_reader")];
const COMMIT_READER_ALLOWED_USE_PREFIXES: &[&str] = &[
    "use std::",
    "use futures::",
    "use gpui::",
    "use crate::protocol",
    "use crate::transport",
    "use super::ReactRoot",
];

struct Source {
    relative: String,
    path: PathBuf,
    text: String,
}

fn source(relative: &str) -> Source {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(relative);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    Source {
        relative: relative.to_owned(),
        path,
        text,
    }
}

fn renderer_sources() -> Vec<Source> {
    RENDERER_CHILD_MODULES
        .iter()
        .map(|name| source(&format!("renderer/{name}")))
        .collect()
}

fn paint_sources() -> Vec<Source> {
    PAINT_CHILD_MODULES
        .iter()
        .map(|name| source(&format!("renderer/paint/{name}")))
        .collect()
}

fn protocol_sources() -> Vec<Source> {
    PROTOCOL_CHILD_MODULES
        .iter()
        .map(|name| source(&format!("protocol/{name}")))
        .collect()
}
fn tree_sources() -> Vec<Source> {
    TREE_CHILD_MODULES
        .iter()
        .map(|name| source(&format!("tree/{name}")))
        .collect()
}

fn fail(source: &Source, line_number: usize, reason: &str, line: &str) -> ! {
    panic!(
        "module boundary violation in {}:{}: {reason}\n  {}",
        source.path.display(),
        line_number,
        line.trim(),
    );
}

fn assert_no_line_containing(source: &Source, needle: &str, reason: &str) {
    for (index, line) in source.text.lines().enumerate() {
        if line.contains(needle) {
            fail(source, index + 1, reason, line);
        }
    }
}

fn assert_contains_line(source: &Source, needle: &str, reason: &str) {
    if source.text.lines().any(|line| line.contains(needle)) {
        return;
    }
    let context = source.text.lines().next().unwrap_or_default();
    fail(source, 1, reason, context);
}

#[test]
fn renderer_and_tree_parents_keep_children_private() {
    let renderer = source("renderer.rs");
    for child in RENDERER_CHILD_MODULES {
        let module = child
            .strip_suffix(".rs")
            .expect("renderer child list entries must be Rust files");
        assert_contains_line(
            &renderer,
            &format!("mod {module}"),
            "renderer child module declaration is missing",
        );
        assert_no_line_containing(
            &renderer,
            &format!("pub mod {module}"),
            "renderer child modules must remain private to renderer",
        );
    }
    assert_contains_line(
        &renderer,
        "mod paint",
        "renderer paint module declaration is missing",
    );
    assert_no_line_containing(
        &renderer,
        "pub mod paint",
        "renderer paint module must remain private to renderer",
    );

    let paint = source("renderer/paint/mod.rs");
    for child in PAINT_CHILD_MODULES {
        let module = child
            .strip_suffix(".rs")
            .expect("paint child list entries must be Rust files");
        assert_contains_line(
            &paint,
            &format!("mod {module}"),
            "paint child module declaration is missing",
        );
        assert_no_line_containing(
            &paint,
            &format!("pub mod {module}"),
            "paint child modules must remain private to paint",
        );
    }

    let tree = source("tree.rs");
    for child in TREE_CHILD_MODULES {
        let module = child
            .strip_suffix(".rs")
            .expect("tree child list entries must be Rust files");
        assert_contains_line(
            &tree,
            &format!("mod {module}"),
            "tree child module declaration is missing",
        );
        assert_no_line_containing(
            &tree,
            &format!("pub mod {module}"),
            "tree child modules must remain private to tree",
        );
    }
}

fn is_public_item(line: &str) -> bool {
    [
        "pub fn ",
        "pub unsafe fn ",
        "pub async fn ",
        "pub struct ",
        "pub enum ",
        "pub trait ",
        "pub type ",
        "pub const ",
        "pub static ",
        "pub use ",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
}

fn is_allowlisted_public_item(source: &Source, line: &str) -> bool {
    ALLOWED_PUBLIC_ITEMS
        .iter()
        .any(|(relative, prefix)| source.relative == *relative && line.starts_with(prefix))
}

#[test]
fn renderer_and_tree_child_exports_are_internal() {
    for child in renderer_sources().into_iter().chain(paint_sources()) {
        for (index, line) in child.text.lines().enumerate() {
            let trimmed = line.trim_start();
            if !is_public_item(trimmed) || is_allowlisted_public_item(&child, trimmed) {
                continue;
            }
            if ALLOWED_INTERNAL_VISIBILITY
                .iter()
                .any(|prefix| trimmed.starts_with(prefix))
            {
                continue;
            }
            fail(
                &child,
                index + 1,
                "renderer child implementation items must use pub(super) or pub(crate)",
                line,
            );
        }
    }
    for child in tree_sources() {
        for (index, line) in child.text.lines().enumerate() {
            let trimmed = line.trim_start();
            if !is_public_item(trimmed)
                || trimmed.starts_with("pub(super) ")
                || trimmed.starts_with("pub(crate) ")
            {
                continue;
            }
            fail(
                &child,
                index + 1,
                "tree child implementation items must use pub(super) or pub(crate)",
                line,
            );
        }
    }
}

#[test]
fn renderer_child_edges_are_parent_scoped() {
    for child in renderer_sources().into_iter().chain(paint_sources()) {
        for (index, line) in child.text.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("use crate::renderer::") {
                fail(
                    &child,
                    index + 1,
                    "renderer children must not import through a crate-level renderer path",
                    line,
                );
            }
            if trimmed.starts_with("use super::")
                && !ALLOWED_RENDERER_SUPER_IMPORTS
                    .iter()
                    .any(|prefix| trimmed.starts_with(prefix))
            {
                fail(
                    &child,
                    index + 1,
                    "renderer child edge is not in the reviewed super:: allowlist",
                    line,
                );
            }
        }
    }
}

#[test]
fn event_emitter_has_no_root_state_dependency() {
    let events = source("renderer/events.rs");
    assert_no_line_containing(
        &events,
        "super::ReactRoot",
        "events must remain a stateless leaf and not import ReactRoot",
    );
    assert_no_line_containing(
        &events,
        ".store",
        "events must remain a stateless leaf and not reach NodeStore",
    );
}

#[test]
fn commit_reader_dependencies_are_whitelisted() {
    let commit_reader = source("renderer/commit_reader.rs");
    for (index, line) in commit_reader.text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("use ")
            && !COMMIT_READER_ALLOWED_USE_PREFIXES
                .iter()
                .any(|prefix| trimmed.starts_with(prefix))
        {
            fail(
                &commit_reader,
                index + 1,
                "commit_reader may only import std/futures/gpui plus protocol, transport, and ReactRoot",
                line,
            );
        }
    }
}

#[test]
fn protocol_wire_types_stay_private() {
    let protocol = source("protocol.rs");
    assert_contains_line(
        &protocol,
        "mod wire;",
        "protocol wire child declaration is missing",
    );
    assert_no_line_containing(
        &protocol,
        "pub mod wire;",
        "protocol wire module must remain private",
    );
    for (index, line) in protocol.text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("pub use wire::") {
            fail(
                &protocol,
                index + 1,
                "protocol wire implementation must not be re-exported",
                line,
            );
        }
        if is_public_item(trimmed) && trimmed.contains("Wire") {
            fail(
                &protocol,
                index + 1,
                "wire types must not appear in protocol public item signatures",
                line,
            );
        }
    }

    for wire in protocol_sources() {
        for (index, line) in wire.text.lines().enumerate() {
            let trimmed = line.trim_start();
            if is_public_item(trimmed) && !trimmed.starts_with("pub(super) ") {
                fail(
                    &wire,
                    index + 1,
                    "protocol wire implementation items must remain private to protocol",
                    line,
                );
            }
        }
    }
}
