/// Collects all expressions in "return position" from a HIR function body.
///
/// This includes explicit `return expr` statements anywhere in the body, as well
/// as tail expressions — recursing through blocks, `if`/`else`, and `match`
/// arms.
use rustc_hir::{Block, Expr, ExprKind, StmtKind};

/// Collect every expression that could be the returned value of `body_expr`.
///
/// The caller receives a list of "leaf" return expressions — the innermost
/// expressions that actually produce the value, after unwinding control-flow
/// wrappers (blocks, if/else, match).
pub fn collect_return_exprs<'tcx>(body_expr: &'tcx Expr<'tcx>) -> Vec<&'tcx Expr<'tcx>> {
    let mut out = Vec::new();
    collect_explicit_returns(body_expr, &mut out);
    collect_tail_exprs(body_expr, &mut out);
    out
}

/// Recurse into all statements and the optional tail expression of a block,
/// looking for explicit `return` expressions.
fn walk_block_returns<'tcx>(block: &'tcx Block<'tcx>, out: &mut Vec<&'tcx Expr<'tcx>>) {
    for stmt in block.stmts {
        if let StmtKind::Expr(e) | StmtKind::Semi(e) = &stmt.kind {
            collect_explicit_returns(e, out);
        }
    }
    if let Some(tail) = block.expr {
        collect_explicit_returns(tail, out);
    }
}

/// Walk the HIR tree rooted at `expr` and collect every explicit `return e`
/// sub-expression's inner value `e`.
fn collect_explicit_returns<'tcx>(expr: &'tcx Expr<'tcx>, out: &mut Vec<&'tcx Expr<'tcx>>) {
    match &expr.kind {
        ExprKind::Ret(Some(inner)) => {
            collect_tail_exprs(inner, out);
        }
        ExprKind::Block(block, _) | ExprKind::Loop(block, _, _, _) => {
            walk_block_returns(block, out);
        }
        ExprKind::If(_, then_block, else_block) => {
            collect_explicit_returns(then_block, out);
            if let Some(else_expr) = else_block {
                collect_explicit_returns(else_expr, out);
            }
        }
        ExprKind::Match(_, arms, _) => {
            for arm in *arms {
                collect_explicit_returns(arm.body, out);
            }
        }
        _ => {}
    }
}

/// Recursively extract "tail" (leaf) expressions — the expressions that
/// actually produce the value at the end of blocks, if/else branches, and
/// match arms.
fn collect_tail_exprs<'tcx>(expr: &'tcx Expr<'tcx>, out: &mut Vec<&'tcx Expr<'tcx>>) {
    match &expr.kind {
        ExprKind::Block(block, _) => {
            if let Some(tail) = block.expr {
                collect_tail_exprs(tail, out);
            }
        }
        ExprKind::If(_, then_block, Some(else_block)) => {
            collect_tail_exprs(then_block, out);
            collect_tail_exprs(else_block, out);
        }
        ExprKind::Match(_, arms, _) => {
            for arm in *arms {
                collect_tail_exprs(arm.body, out);
            }
        }
        _ => {
            out.push(expr);
        }
    }
}
