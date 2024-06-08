use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::{quote_ident, ExprFactory},
    },
};

use crate::constants::{
    GLOBAL, HELPER_AS_WILDCARD_NAME, MODULE, MODULE_EXTERNAL_NAME, MODULE_HELPER_NAME,
    MODULE_IMPORT_NAME, MODULE_REQUIRE_NAME,
};

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

/// Returns an statement that require module from global.
///
/// eg. `global.__modules.external('module_src')`
pub fn external_module_from_global(module_src: &String) -> Expr {
    obj_member_expr(
        obj_member_expr(quote_ident!(GLOBAL).into(), quote_ident!(MODULE).into()),
        quote_ident!(MODULE_EXTERNAL_NAME),
    )
    .as_call(
        DUMMY_SP,
        vec![Expr::Lit(Lit::Str(Str::from(module_src.as_str()))).as_arg()],
    )
}

/// Create unique module identifier and returns a statement that import default value from global.
///
/// eg. `const ident = {module_ident}.default`
/// eg. `import ident from "module_src"`
pub fn create_default_import_stmt(
    module_src: &String,
    ident: &Ident,
    runtime_module_ident: Option<&Ident>,
) -> ModuleItem {
    if let Some(runtime_module_ident) = runtime_module_ident {
        decl_var_and_assign_stmt(
            &ident,
            obj_member_expr(runtime_module_ident.clone().into(), quote_ident!("default")),
        )
        .into()
    } else {
        ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
            span: DUMMY_SP,
            specifiers: vec![ImportDefaultSpecifier {
                span: DUMMY_SP,
                local: ident.clone(),
            }
            .into()],
            src: Str::from(module_src.clone()).into(),
            type_only: false,
            with: None,
            phase: ImportPhase::Evaluation,
        }))
    }
}

/// Create unique module identifier and returns a statement that import named value from global.
///
/// eg. `const ident = {module_ident}.ident`
/// eg. `import { ident } from "module_src"`
pub fn create_named_import_stmt(
    module_src: &String,
    ident: &Ident,
    runtime_module_ident: Option<&Ident>,
    imported: &Option<Ident>,
) -> ModuleItem {
    if let Some(runtime_module_ident) = runtime_module_ident {
        decl_var_and_assign_stmt(
            &ident,
            obj_member_expr(
                runtime_module_ident.clone().into(),
                quote_ident!(imported.clone().unwrap_or(ident.clone()).sym),
            ),
        )
        .into()
    } else {
        ModuleDecl::Import(ImportDecl {
            span: DUMMY_SP,
            specifiers: vec![ImportNamedSpecifier {
                span: DUMMY_SP,
                local: ident.clone(),
                imported: imported.clone().and_then(|imported_ident| {
                    Some(ModuleExportName::Ident(imported_ident.into()))
                }),
                is_type_only: false,
            }
            .into()],
            src: Str::from(module_src.clone()).into(),
            type_only: false,
            with: None,
            phase: ImportPhase::Evaluation,
        })
        .into()
    }
}

/// Create unique module identifier and returns a statement that import namespaced value from global.
///
/// eg. `const ident = global.__modules.helpers.asWildcard(module_ident)`
/// eg. `import * as ident from "module_src"`
pub fn create_namespace_import_stmt(
    module_src: &String,
    ident: &Ident,
    runtime_module_ident: Option<&Ident>,
) -> ModuleItem {
    if let Some(runtime_module_ident) = runtime_module_ident {
        decl_var_and_assign_stmt(
            &ident,
            Expr::Call(CallExpr {
                span: DUMMY_SP,
                type_args: None,
                callee: Callee::Expr(Box::new(obj_member_expr(
                    obj_member_expr(
                        obj_member_expr(quote_ident!(GLOBAL).into(), quote_ident!(MODULE).into()),
                        quote_ident!(MODULE_HELPER_NAME),
                    ),
                    quote_ident!(HELPER_AS_WILDCARD_NAME),
                ))),
                args: vec![runtime_module_ident.clone().as_arg()],
            }),
        )
        .into()
    } else {
        ModuleDecl::Import(ImportDecl {
            span: DUMMY_SP,
            src: Str::from(module_src.clone()).into(),
            type_only: false,
            with: None,
            phase: ImportPhase::Evaluation,
            specifiers: vec![ImportStarAsSpecifier {
                span: DUMMY_SP,
                local: ident.clone(),
            }
            .into()],
        })
        .into()
    }
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
