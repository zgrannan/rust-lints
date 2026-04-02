#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

mod return_collector;

use rustc_hir::def::{CtorOf, DefKind, Res};
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_hir::{Expr, ExprKind, ItemKind, Node, QPath};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::{self, TyCtxt, Visibility};
use rustc_span::Symbol;

use return_collector::collect_return_exprs;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Detects non-pub trait methods whose return type is an enum, where every
    /// implementation returns the same single-field variant. In that case the
    /// enum wrapper is unnecessary and the method could return the inner type
    /// directly.
    ///
    /// ### Why is this bad?
    ///
    /// An enum return type suggests that different implementations may produce
    /// different variants. When they all produce the same one, the enum adds
    /// indirection without value — callers must unwrap a variant that is always
    /// the same.
    ///
    /// ### Example
    ///
    /// ```rust
    /// enum Output { Text(String), Number(i64) }
    ///
    /// trait Render {
    ///     fn render(&self) -> Output;
    /// }
    ///
    /// // Every impl returns Output::Text(...)
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// trait Render {
    ///     fn render(&self) -> String;
    /// }
    /// ```
    pub REDUNDANT_ENUM_VARIANT,
    Warn,
    "trait method always returns the same enum variant across all impls"
}

impl<'tcx> LateLintPass<'tcx> for RedundantEnumVariant {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        let tcx = cx.tcx;

        for item_id in tcx.hir_crate_items(()).free_items() {
            let item = tcx.hir_item(item_id);

            let ItemKind::Trait(_, _, _, _, _, _, trait_item_ids) = &item.kind else {
                continue;
            };

            let trait_def_id = item.owner_id.to_def_id();

            if tcx.visibility(trait_def_id) == Visibility::Public {
                continue;
            }

            for trait_item_id in *trait_item_ids {
                let method_def_id = trait_item_id.owner_id.to_def_id();
                let assoc_item = tcx.associated_item(method_def_id);
                if !matches!(assoc_item.kind, ty::AssocKind::Fn { .. }) {
                    continue;
                }
                check_trait_method(cx, trait_def_id, method_def_id, assoc_item.name());
            }
        }
    }
}

/// Inspect a single trait method: warn if every impl returns the same
/// single-field enum variant.
fn check_trait_method(
    cx: &LateContext<'_>,
    trait_def_id: DefId,
    method_def_id: DefId,
    method_name: Symbol,
) {
    let tcx = cx.tcx;

    let fn_sig = tcx.fn_sig(method_def_id).skip_binder().skip_binder();
    let ret_ty = fn_sig.output();

    let ty::Adt(adt_def, substs) = ret_ty.kind() else {
        return;
    };
    if !adt_def.is_enum() {
        return;
    }

    let impls = tcx.trait_impls_of(trait_def_id);
    let all_impl_def_ids: Vec<DefId> = impls
        .blanket_impls()
        .iter()
        .chain(impls.non_blanket_impls().values().flat_map(|v| v.iter()))
        .copied()
        .collect();

    if all_impl_def_ids.is_empty() {
        return;
    }

    let mut seen_variant: Option<DefId> = None;

    for &impl_def_id in &all_impl_def_ids {
        let body_owner = if let Some(local) = find_impl_method(tcx, impl_def_id, method_name) {
            local
        } else {
            let has_default = tcx
                .provided_trait_methods(trait_def_id)
                .any(|m| m.name() == method_name);
            if !has_default {
                return;
            }
            method_def_id.expect_local()
        };

        if !check_body_variant(cx, body_owner, &mut seen_variant) {
            return;
        }
    }

    let Some(variant_def_id) = seen_variant else {
        return;
    };

    let Some(variant) = adt_def
        .variants()
        .iter()
        .find(|v| v.def_id == variant_def_id)
    else {
        return;
    };

    if variant.fields.len() != 1 {
        return;
    }

    let field_ty = variant.fields.iter().next().unwrap().ty(tcx, substs);
    let variant_name = variant.name;
    let span = tcx.def_span(method_def_id);

    tcx.node_span_lint(
        REDUNDANT_ENUM_VARIANT,
        tcx.local_def_id_to_hir_id(method_def_id.expect_local()),
        span,
        |diag| {
            diag.help(format!(
                "every impl returns `{variant_name}(...)` — \
                 consider changing the return type to `{field_ty}`",
            ));
        },
    );
}

/// Find the `LocalDefId` of the method named `method_name` inside the impl
/// identified by `impl_def_id`. Returns `None` when the impl does not override
/// this method (relying on the default body).
fn find_impl_method(
    tcx: TyCtxt<'_>,
    impl_def_id: DefId,
    method_name: Symbol,
) -> Option<LocalDefId> {
    let impl_local = impl_def_id.as_local()?;
    let Node::Item(item) = tcx.hir_node_by_def_id(impl_local) else {
        return None;
    };
    let ItemKind::Impl(impl_block) = &item.kind else {
        return None;
    };
    for item_id in impl_block.items {
        let assoc = tcx.associated_item(item_id.owner_id.to_def_id());
        if assoc.name() == method_name {
            return Some(item_id.owner_id.def_id);
        }
    }
    None
}

/// Check that every return-position expression in the body constructs the same
/// enum variant. Updates `seen_variant` and returns `false` on mismatch or
/// unrecognised expressions.
fn check_body_variant(
    cx: &LateContext<'_>,
    local_def_id: LocalDefId,
    seen_variant: &mut Option<DefId>,
) -> bool {
    let body = cx.tcx.hir_body_owned_by(local_def_id);
    let return_exprs = collect_return_exprs(body.value);

    if return_exprs.is_empty() {
        return false;
    }

    for expr in return_exprs {
        let Some(variant_did) = variant_def_id_of_expr(cx.tcx, expr) else {
            return false;
        };
        match *seen_variant {
            None => *seen_variant = Some(variant_did),
            Some(prev) if prev != variant_did => return false,
            _ => {}
        }
    }

    true
}

/// If `expr` constructs an enum variant (tuple or struct style), return the
/// variant's `DefId` (not the constructor's).
fn variant_def_id_of_expr(tcx: TyCtxt<'_>, expr: &Expr<'_>) -> Option<DefId> {
    match &expr.kind {
        ExprKind::Call(func, _) => {
            if let ExprKind::Path(QPath::Resolved(_, path)) = &func.kind
                && let Res::Def(DefKind::Ctor(CtorOf::Variant, _), ctor_def_id) = path.res
            {
                return Some(tcx.parent(ctor_def_id));
            }
            None
        }
        ExprKind::Struct(qpath, _, _) => {
            if let QPath::Resolved(_, path) = qpath
                && let Res::Def(DefKind::Variant, def_id) = path.res
            {
                return Some(def_id);
            }
            None
        }
        _ => None,
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
