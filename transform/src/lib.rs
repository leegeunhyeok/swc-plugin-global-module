mod constants;
mod module_collector_esm;
mod utils;

use constants::{
    ESM_API_NAME, GLOBAL, HELPER_AS_WILDCARD_NAME, MODULE, MODULE_HELPER_NAME, MODULE_REGISTRY_NAME,
};
use module_collector_esm::{EsModuleCollector, ExportModule, ImportModule, ModuleType};
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::{private_ident, quote_ident, ExprFactory},
        visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
    },
};
use utils::{decl_var_and_assign_stmt, obj_lit, obj_member_expr};

pub struct GlobalModuleTransformer {
    module_name: String,
    runtime_module: bool,
    import_paths: Option<HashMap<String, String>>,
    import_idents: BTreeMap<String, Ident>,
    normalize_regex: Regex,
}

impl GlobalModuleTransformer {
    fn new(
        module_name: String,
        runtime_module: bool,
        import_paths: Option<HashMap<String, String>>,
    ) -> Self {
        GlobalModuleTransformer {
            module_name,
            runtime_module,
            import_paths,
            import_idents: BTreeMap::new(),
            normalize_regex: Regex::new(r"[^a-zA-Z0-9]").unwrap(),
        }
    }

    /// Find actual module path from `import_paths`
    fn to_actual_path(&self, module_src: String) -> String {
        if let Some(actual_path) = self
            .import_paths
            .as_ref()
            .and_then(|import_paths| import_paths.get(&module_src))
        {
            return actual_path.clone();
        }
        module_src
    }

    /// Returns an statement that import module from global and assign it.
    ///
    /// eg. `const __mod = global.__modules.registry[module_id]`
    fn get_global_import_stmt(&self, ident: &Ident, module_id: &str) -> Stmt {
        decl_var_and_assign_stmt(
            ident,
            Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: obj_member_expr(
                    obj_member_expr(quote_ident!(GLOBAL).into(), quote_ident!(MODULE).into()),
                    quote_ident!(MODULE_REGISTRY_NAME),
                )
                .into(),
                prop: MemberProp::Computed(ComputedPropName {
                    span: DUMMY_SP,
                    expr: Box::new(Expr::Lit(Lit::Str(Str::from(module_id)))),
                }),
            }),
        )
    }

    /// Returns a cached module ident.
    fn get_module_ident(&mut self, module_src: &String) -> &Ident {
        let module_path = self.to_actual_path(module_src.to_string());
        self.import_idents
            .entry(module_path)
            .or_insert(private_ident!(self
                .normalize_regex
                .replace_all(format!("_{module_src}").as_str(), "_")
                .to_string()))
    }

    /// Create unique module identifier and returns a statement that import default value from global.
    ///
    /// eg. `const ident = {module_ident}.default`
    /// eg. `import ident from "module_src"`
    fn create_default_import_stmt(&mut self, module_src: &String, ident: &Ident) -> ModuleItem {
        if self.runtime_module {
            let module_ident = self.get_module_ident(&module_src);
            decl_var_and_assign_stmt(
                &ident,
                obj_member_expr(module_ident.clone().into(), quote_ident!("default")),
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
            }))
        }
    }

    /// Create unique module identifier and returns a statement that import named value from global.
    ///
    /// eg. `const ident = {module_ident}.ident`
    /// eg. `import { ident } from "module_src"`
    fn create_named_import_stmt(
        &mut self,
        module_src: &String,
        ident: &Ident,
        imported: &Option<Ident>,
    ) -> ModuleItem {
        if self.runtime_module {
            let module_ident = self.get_module_ident(&module_src);
            decl_var_and_assign_stmt(
                &ident,
                obj_member_expr(
                    module_ident.clone().into(),
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
            })
            .into()
        }
    }

    /// Create unique module identifier and returns a statement that import namespaced value from global.
    ///
    /// eg. `const ident = global.__modules.helpers.asWildcard(module_ident)`
    /// eg. `import * as ident from "module_src"`
    fn create_namespace_import_stmt(&mut self, module_src: &String, ident: &Ident) -> ModuleItem {
        if self.runtime_module {
            let module_ident = self.get_module_ident(&module_src);
            decl_var_and_assign_stmt(
                &ident,
                Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    type_args: None,
                    callee: Callee::Expr(Box::new(obj_member_expr(
                        obj_member_expr(
                            obj_member_expr(
                                quote_ident!(GLOBAL).into(),
                                quote_ident!(MODULE).into(),
                            ),
                            quote_ident!(MODULE_HELPER_NAME),
                        ),
                        quote_ident!(HELPER_AS_WILDCARD_NAME),
                    ))),
                    args: vec![module_ident.clone().as_arg()],
                }),
            )
            .into()
        } else {
            ModuleDecl::Import(ImportDecl {
                span: DUMMY_SP,
                src: Str::from(module_src.clone()).into(),
                type_only: false,
                with: None,
                specifiers: vec![ImportStarAsSpecifier {
                    span: DUMMY_SP,
                    local: ident.clone(),
                }
                .into()],
            })
            .into()
        }
    }

    fn convert_esm_import(&mut self, imports: &Vec<ImportModule>) -> Vec<ModuleItem> {
        let mut stmts = Vec::with_capacity(imports.len());

        imports.iter().for_each(
            |ImportModule {
                 ident,
                 imported,
                 module_src,
                 module_type,
             }| match module_type {
                ModuleType::Default | ModuleType::DefaultAsNamed => {
                    stmts.push(self.create_default_import_stmt(module_src, ident).into());
                }
                ModuleType::Named => stmts.push(
                    self.create_named_import_stmt(module_src, ident, imported)
                        .into(),
                ),
                ModuleType::NamespaceOrAll => {
                    stmts.push(self.create_namespace_import_stmt(module_src, ident).into())
                }
            },
        );

        if self.runtime_module {
            let mut import_ident_stmts = self
                .import_idents
                .iter()
                .map(|(src, ident)| self.get_global_import_stmt(ident, src).into())
                .collect::<Vec<ModuleItem>>();

            import_ident_stmts.extend(stmts);
            import_ident_stmts
        } else {
            stmts
        }
    }

    fn convert_esm_export(&mut self, exports: &Vec<ExportModule>) -> Vec<ModuleItem> {
        let mut stmts = Vec::with_capacity(exports.len());
        let mut export_props = Vec::new();
        let mut export_all_props = Vec::new();

        exports.into_iter().for_each(
            |ExportModule {
                 ident,
                 as_ident,
                 module_type,
             }| {
                match module_type {
                    ModuleType::Default | ModuleType::DefaultAsNamed => {
                        export_props.push(
                            Prop::KeyValue(KeyValueProp {
                                key: quote_ident!("default").into(),
                                value: ident.clone().into(),
                            })
                            .into(),
                        );
                    }
                    ModuleType::Named => {
                        export_props.push(
                            if let Some(renamed_ident) =
                                as_ident.as_ref().filter(|&id| id.sym != ident.sym)
                            {
                                Prop::KeyValue(KeyValueProp {
                                    key: quote_ident!(renamed_ident.sym.as_str()).into(),
                                    value: ident.clone().into(),
                                })
                                .into()
                            } else {
                                Prop::Shorthand(ident.clone()).into()
                            },
                        );
                    }
                    ModuleType::NamespaceOrAll => export_all_props.push(ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Ident(ident.clone())),
                    }),
                }
            },
        );

        let mut args = vec![
            self.module_name.as_str().as_arg(),
            obj_lit(Some(export_props)).as_arg(),
        ];
        args.extend(export_all_props);
        stmts.push(
            obj_member_expr(
                obj_member_expr(quote_ident!(GLOBAL).into(), quote_ident!(MODULE).into()),
                quote_ident!(ESM_API_NAME),
            )
            .as_call(DUMMY_SP, args)
            .into_stmt()
            .into(),
        );

        stmts
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        let mut esm_collector = EsModuleCollector::new(self.runtime_module);
        module.visit_mut_with(&mut esm_collector);

        module
            .body
            .splice(..0, self.convert_esm_import(&esm_collector.imports));

        module
            .body
            .extend(self.convert_esm_export(&esm_collector.exports));
    }
}

pub fn global_module(
    module_name: String,
    runtime_module: bool,
    import_paths: Option<HashMap<String, String>>,
) -> impl VisitMut + Fold {
    as_folder(GlobalModuleTransformer::new(
        module_name,
        runtime_module,
        import_paths,
    ))
}
