use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::{quote_ident, ExprFactory},
    },
};

use crate::constants::{GLOBAL, MODULE, MODULE_IMPORT_NAME, MODULE_REQUIRE_NAME};

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

/// Returns an statement that import module from global.
///
/// eg. `global.__modules.import('module_id')`
pub fn import_module_from_global(src: &String) -> Expr {
    obj_member_expr(
        obj_member_expr(quote_ident!(GLOBAL).into(), quote_ident!(MODULE).into()),
        quote_ident!(MODULE_IMPORT_NAME),
    )
    .as_call(
        DUMMY_SP,
        vec![Expr::Lit(Lit::Str(Str::from(src.as_str()))).as_arg()],
    )
}

/// Returns an statement that require module from global.
///
/// eg. `global.__modules.require('module_id')`
pub fn require_module_from_global(src: &String) -> Expr {
    obj_member_expr(
        obj_member_expr(quote_ident!(GLOBAL).into(), quote_ident!(MODULE).into()),
        quote_ident!(MODULE_REQUIRE_NAME),
    )
    .as_call(
        DUMMY_SP,
        vec![Expr::Lit(Lit::Str(Str::from(src.as_str()))).as_arg()],
    )
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
