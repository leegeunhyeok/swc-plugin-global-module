use swc_core::{
    common::DUMMY_SP,
    ecma::{ast::*, utils::quote_ident},
};

use crate::constants::{GLOBAL, MODULE, MODULE_REGISTRY_NAME};

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

/// Returns an statement that import module from global and assign it.
///
/// eg. `global.__modules.registry[module_id]`
pub fn get_module_from_global(src: &String) -> Expr {
    Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: obj_member_expr(
            obj_member_expr(quote_ident!(GLOBAL).into(), quote_ident!(MODULE).into()),
            quote_ident!(MODULE_REGISTRY_NAME),
        )
        .into(),
        prop: MemberProp::Computed(ComputedPropName {
            span: DUMMY_SP,
            expr: Box::new(Expr::Lit(Lit::Str(Str::from(src.as_str())))),
        }),
    })
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
