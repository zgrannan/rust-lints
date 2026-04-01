#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;

use rustc_hir::{HirId, Item, ItemKind, Node, Path};
use rustc_lint::{LateContext, LateLintPass};

/// Paths with more than this many segments trigger a warning.
const MAX_SEGMENTS: usize = 3;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Checks for qualified paths with more than three segments used outside of
    /// `use` statements (e.g. `a::b::c::d::e`).
    ///
    /// ### Why is this bad?
    ///
    /// Long qualified paths reduce readability. Prefer importing a prefix with
    /// `use` and then referring to the item with a shorter path.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let v: std::collections::hash_map::HashMap<i32, i32> = Default::default();
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// use std::collections::hash_map;
    /// let v: hash_map::HashMap<i32, i32> = Default::default();
    /// ```
    pub LONG_PATH,
    Warn,
    "qualified paths with more than three segments should use an import"
}

impl<'tcx> LateLintPass<'tcx> for LongPath {
    fn check_path(&mut self, cx: &LateContext<'tcx>, path: &Path<'tcx>, hir_id: HirId) {
        if path.segments.len() <= MAX_SEGMENTS {
            return;
        }

        if is_inside_use(cx, hir_id) {
            return;
        }

        let import_len = path.segments.len() - MAX_SEGMENTS + 1;
        let import_path: Vec<&str> = path.segments[..import_len]
            .iter()
            .map(|seg| seg.ident.as_str())
            .collect();
        let suggestion = import_path.join("::");

        cx.tcx.node_span_lint(LONG_PATH, hir_id, path.span, |diag| {
            diag.help(format!("consider importing: `use {suggestion};`"));
        });
    }
}

/// Walk up the HIR parent chain to determine whether `hir_id` is inside a `use`
/// item.
fn is_inside_use(cx: &LateContext<'_>, hir_id: HirId) -> bool {
    let mut current = hir_id;
    loop {
        let node = cx.tcx.hir_node(current);
        match node {
            Node::Item(Item {
                kind: ItemKind::Use(..),
                ..
            }) => return true,
            Node::Crate(..) => return false,
            _ => {}
        }
        current = cx.tcx.parent_hir_id(current);
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
