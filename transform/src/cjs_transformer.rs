use swc_core::{
    common::{util::take::Take, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{private_ident, quote_ident, ExprFactory},
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};

use crate::{
    constants::{CJS_API_NAME, GLOBAL, MODULE},
    helpers::{decl_var_and_assign_stmt, obj_member_expr, require_module_from_global},
    module_resolver::ModuleResolver,
};

#[derive(Debug)]
pub struct Require {
    pub ident: Ident,
    pub module_src: String,
}

#[derive(Debug)]
pub struct Exports {
    pub expr: Expr,
    pub name: String,
}

pub struct CommonJsTransformer<'a> {
    resolver: &'a ModuleResolver,
    module_name: String,
    runtime_module: bool,
    cjs_boundary_ident: Ident,
    exported: i32,
}

impl<'a> CommonJsTransformer<'a> {
    pub fn new(resolver: &'a ModuleResolver, module_name: String, runtime_module: bool) -> Self {
        CommonJsTransformer {
            resolver,
            module_name,
            runtime_module,
            cjs_boundary_ident: private_ident!("__cjs"),
            exported: 0,
        }
    }

    /// Returns an expression that create new CommonJS boundary.
    ///
    /// eg. `const boundary = global.__modules.cjs("module_src")`
    fn get_cjs_boundary(&mut self) -> Expr {
        obj_member_expr(
            obj_member_expr(quote_ident!(GLOBAL).into(), quote_ident!(MODULE).into()),
            quote_ident!(CJS_API_NAME).into(),
        )
        .as_call(DUMMY_SP, vec![self.module_name.as_str().as_arg()])
    }
}

impl VisitMut for CommonJsTransformer<'_> {
    noop_visit_mut_type!();

    fn visit_mut_module_items(&mut self, stmts: &mut Vec<ModuleItem>) {
        stmts.visit_mut_children_with(self);
        if self.exported > 0 {
            let cjs_boundary_expr = self.get_cjs_boundary();
            stmts.insert(
                0,
                decl_var_and_assign_stmt(&self.cjs_boundary_ident, cjs_boundary_expr).into(),
            );
        }
    }

    fn visit_mut_stmt(&mut self, stmt: &mut Stmt) {
        stmt.visit_mut_children_with(self);
        match stmt {
            Stmt::Expr(ExprStmt { expr, .. }) => {
                if expr.is_invalid() {
                    stmt.take();
                }
            }
            _ => {}
        }
    }

    fn visit_mut_exprs(&mut self, exprs: &mut Vec<Box<Expr>>) {
        exprs.visit_mut_children_with(self);
        exprs.retain(|expr| if expr.is_invalid() { false } else { true });
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);
        match expr {
            // Requires
            // `require('...')`
            // `wrapper(require(...))`
            Expr::Call(CallExpr {
                args,
                callee: Callee::Expr(callee_expr),
                type_args: None,
                ..
            }) if self.runtime_module
                && args.len() == 1
                && callee_expr.is_ident_ref_to("require") =>
            {
                let src = match args.first().unwrap() {
                    ExprOrSpread {
                        spread: None,
                        expr: arg_expr,
                    } => match **arg_expr {
                        Expr::Lit(Lit::Str(ref src)) => src.value.to_string(),
                        _ => return,
                    },
                    _ => return,
                };
                *expr = require_module_from_global(
                    &self
                        .resolver
                        .to_actual_path(&src, false)
                        .unwrap_or(src.to_string()),
                );
            }
            // Exports
            // `module.exports = foo`
            // `exports.bar = baz`
            Expr::Assign(AssignExpr {
                op: AssignOp::Assign,
                left: left_pat_or_expr,
                right: right_expr,
                ..
            }) => {
                if let Some(left_expr) = left_pat_or_expr.as_expr() {
                    match left_expr {
                        Expr::Member(MemberExpr {
                            obj,
                            prop: MemberProp::Ident(prop_ident),
                            ..
                        }) => {
                            let export_name = if obj.is_ident_ref_to("exports") {
                                prop_ident.sym.as_str()
                            } else if obj.is_ident_ref_to("module") && prop_ident.sym == "exports" {
                                "default"
                            } else {
                                return;
                            };

                            self.exported += 1;
                            *expr = right_expr
                                .clone()
                                .make_assign_to(
                                    AssignOp::Assign,
                                    obj_member_expr(
                                        obj_member_expr(
                                            Expr::Ident(self.cjs_boundary_ident.clone()),
                                            quote_ident!("exports"),
                                        ),
                                        quote_ident!(export_name),
                                    )
                                    .as_pat_or_expr(),
                                )
                                .make_assign_to(
                                    AssignOp::Assign,
                                    left_expr.clone().as_pat_or_expr(),
                                );
                        }
                        _ => {}
                    };
                }
            }
            _ => {}
        }
    }
}
