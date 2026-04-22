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
    /// Checks for `.ok().is_some()` on a `Result`.
    ///
    /// ### Why is this bad?
    ///
    /// `.ok()` converts a `Result<T, E>` into an `Option<T>`, discarding the
    /// error. Calling `.is_some()` on that immediately throws away the value
    /// too. The direct equivalent — `.is_ok()` — expresses the intent more
    /// clearly and avoids the intermediate `Option`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let ok: bool = some_result.ok().is_some();
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// let ok: bool = some_result.is_ok();
    /// ```
    pub OK_IS_SOME,
    Warn,
    "use `.is_ok()` instead of `.ok().is_some()`"
}

impl<'tcx> LateLintPass<'tcx> for OkIsSome {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &Expr<'tcx>) {
        if expr.span.from_expansion() {
            return;
        }

        let ExprKind::MethodCall(is_some_method, ok_call, _, _) = &expr.kind else {
            return;
        };

        if is_some_method.ident.name.as_str() != "is_some" {
            return;
        }

        let ok_call_ty = cx.typeck_results().expr_ty(ok_call);
        let ty::TyKind::Adt(adt_def, _) = ok_call_ty.kind() else {
            return;
        };
        if !cx.tcx.is_diagnostic_item(sym::Option, adt_def.did()) {
            return;
        }

        if ok_call.span.from_expansion() {
            return;
        }

        let ExprKind::MethodCall(ok_method, _, _, _) = &ok_call.kind else {
            return;
        };

        if ok_method.ident.name.as_str() != "ok" {
            return;
        }

        cx.tcx.node_span_lint(OK_IS_SOME, expr.hir_id, expr.span, |diag| {
            diag.help("use `.is_ok()` instead");
        });
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
