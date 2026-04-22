#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty;
use rustc_span::sym;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Checks for `.collect().join(sep)` on iterators.
    ///
    /// ### Why is this bad?
    ///
    /// Collecting into a temporary `Vec` just to call `join` on it allocates
    /// unnecessarily. The `itertools::Itertools::join` method performs the same
    /// operation directly on the iterator without the intermediate allocation.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let s = words.iter().map(|w| w.as_str()).collect::<Vec<_>>().join(", ");
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// use itertools::Itertools;
    /// let s = words.iter().map(|w| w.as_str()).join(", ");
    /// ```
    pub COLLECT_JOIN,
    Warn,
    "use `itertools::Itertools::join` instead of `.collect().join(sep)`"
}

impl<'tcx> LateLintPass<'tcx> for CollectJoin {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &Expr<'tcx>) {
        if expr.span.from_expansion() {
            return;
        }

        let ExprKind::MethodCall(join_method, receiver, _, _) = &expr.kind else {
            return;
        };

        if join_method.ident.name.as_str() != "join" {
            return;
        }

        let recv_ty = cx.typeck_results().expr_ty(receiver);
        let ty::TyKind::Adt(adt_def, _) = recv_ty.kind() else {
            return;
        };
        if !cx.tcx.is_diagnostic_item(sym::Vec, adt_def.did()) {
            return;
        }

        if receiver.span.from_expansion() {
            return;
        }

        let ExprKind::MethodCall(collect_method, _, _, _) = &receiver.kind else {
            return;
        };

        if collect_method.ident.name.as_str() != "collect" {
            return;
        }

        cx.tcx.node_span_lint(COLLECT_JOIN, expr.hir_id, expr.span, |diag| {
            diag.help(
                "use `itertools::Itertools::join` to avoid allocating a temporary `Vec`",
            );
        });
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
