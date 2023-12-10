use swc_core::{common::DUMMY_SP, ecma::ast::*};

/// Returns an object member expression.
///
/// eg. `obj.prop`
pub fn obj_member_expr(obj: Expr, prop: Ident) -> Expr {
    Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: obj.into(),
        prop: prop.into(),
    })
}

/// Returns an assign expression with declare variable statement.
///
/// eg. `const name = expr`
pub fn decl_var_and_assign_stmt(ident: &Ident, init: Expr) -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Const,
        declare: false,
        decls: vec![VarDeclarator {
            span: ident.span,
            name: ident.clone().into(),
            init: Some(init.into()),
            definite: false,
        }],
    })))
}

/// Returns an object literal expression.
///
/// eg. `{ props }`
pub fn obj_lit(props: Option<Vec<PropOrSpread>>) -> Expr {
    ObjectLit {
        span: DUMMY_SP,
        props: props.unwrap_or(Vec::new()),
    }
    .into()
}

/// Check `ModuleDecl` is invalid.
pub fn is_invalid_module_decl(module_decl: &ModuleDecl) -> bool {
    if let ModuleDecl::Import(ImportDecl {
        src, specifiers, ..
    }) = module_decl
    {
        src.is_empty() && specifiers.is_empty()
    } else {
        false
    }
}
