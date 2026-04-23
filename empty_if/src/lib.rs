#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

extern crate rustc_hir;

use rustc_hir::{BinOpKind, Block, Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Checks for `if` expressions whose then-block is empty.
    ///
    /// ### Why is this bad?
    ///
    /// An empty then-block is always a code smell:
    ///
    /// - `if cond {}` with no else is dead code and should be removed.
    /// - `if cond {} else { body }` should be written as `if !cond { body }`,
    ///   which makes the condition match the action and is easier to read.
    ///
    /// ### Example
    ///
    /// ```rust
    /// if x.is_some() {} else {
    ///     handle_none();
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// if x.is_none() {
    ///     handle_none();
    /// }
    /// ```
    pub EMPTY_IF,
    Warn,
    "if expression with an empty then-block"
}

impl<'tcx> LateLintPass<'tcx> for EmptyIf {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &Expr<'tcx>) {
        if expr.span.from_expansion() {
            return;
        }

        let ExprKind::If(cond, then_expr, else_opt) = &expr.kind else {
            return;
        };

        // Negating a let-binding condition (if let / let chains) is non-trivial.
        if has_let_expr(cond) {
            return;
        }

        let ExprKind::Block(block, _) = &then_expr.kind else {
            return;
        };

        if !block.stmts.is_empty() || block.expr.is_some() {
            return;
        }

        if block_has_comment(cx, block) {
            return;
        }

        let help = if else_opt.is_some() {
            "negate the condition and replace the empty then-block with the else body"
        } else {
            "the then-block is empty and there is no else; remove the entire `if`"
        };

        cx.tcx.node_span_lint(EMPTY_IF, expr.hir_id, expr.span, |diag| {
            diag.help(help);
        });
    }
}

/// Returns `true` if the block's source text contains a comment (`//` or `/*`).
///
/// Comments are stripped before HIR construction, so source text is the only
/// way to detect them.
fn block_has_comment(cx: &LateContext<'_>, block: &Block<'_>) -> bool {
    cx.tcx
        .sess
        .source_map()
        .span_to_snippet(block.span)
        .map_or(false, |s| s.contains("//") || s.contains("/*"))
}

/// Returns `true` if `expr` is or contains a `let` binding, i.e. originates
/// from an `if let` or a let-chain (`cond && let pat = expr`).
fn has_let_expr(expr: &Expr<'_>) -> bool {
    match &expr.kind {
        ExprKind::Let(..) => true,
        ExprKind::Binary(op, lhs, rhs) if op.node == BinOpKind::And => {
            has_let_expr(lhs) || has_let_expr(rhs)
        }
        _ => false,
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
