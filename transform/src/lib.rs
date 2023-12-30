mod cjs_transformer;
mod constants;
mod esm_collector;
mod helpers;
mod module_resolver;

use cjs_transformer::CommonJsTransformer;
use constants::{ESM_API_NAME, GLOBAL, MODULE, MODULE_EXTERNAL_NAME};
use esm_collector::{EsModuleCollector, ExportModule, ImportModule, ModuleType};
use helpers::{
    create_default_import_stmt, create_named_import_stmt, create_namespace_import_stmt,
    decl_var_and_assign_stmt, external_module_from_global, import_module_from_global, obj_lit,
    obj_member_expr,
};
use module_resolver::ModuleResolver;
use std::collections::HashMap;
use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::{private_ident, quote_ident, ExprFactory},
        visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
    },
};

pub struct GlobalModuleTransformer {
    module_id: String,
    runtime_module: bool,
    external_flags: HashMap<String, bool>,
    resolver: ModuleResolver,
}

impl GlobalModuleTransformer {
    fn new(
        module_id: String,
        runtime_module: bool,
        external_pattern: Option<String>,
        module_ids: Option<HashMap<String, String>>,
    ) -> Self {
        GlobalModuleTransformer {
            module_id,
            runtime_module,
            external_flags: Default::default(),
            resolver: ModuleResolver::new(external_pattern, module_ids),
        }
    }

    fn register_external_module(&mut self, stmts: &mut Vec<ModuleItem>, src: &String) -> bool {
        if !self.resolver.is_external(src) {
            false
        } else if let Some(_) = self.external_flags.get(src) {
            // Already registered.
            true
        } else {
            self.external_flags.insert(src.clone(), true);
            let external_ident = private_ident!("__external");

            // import * as __external from 'src';
            // global.__modules.external('src', __external);
            stmts.push(create_namespace_import_stmt(src, &external_ident, None));
            stmts.push(
                obj_member_expr(
                    obj_member_expr(quote_ident!(GLOBAL).into(), quote_ident!(MODULE).into()),
                    quote_ident!(MODULE_EXTERNAL_NAME),
                )
                .as_call(
                    DUMMY_SP,
                    vec![Expr::from(src.as_str()).as_arg(), external_ident.as_arg()],
                )
                .into_stmt()
                .into(),
            );
            true
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
                 as_export,
             }| {
                if !self.runtime_module
                    && !*as_export
                    && self.register_external_module(&mut stmts, module_src)
                {
                    return;
                }

                if self.runtime_module || (!self.runtime_module && *as_export) {
                    let runtime_module_ident: Option<&Ident> =
                        if self.runtime_module {
                            Some(self.resolver.get_ident_by_src(
                                module_src,
                                self.resolver.is_external(module_src),
                            ))
                        } else {
                            None
                        };

                    stmts.push(
                        match module_type {
                            ModuleType::Default | ModuleType::DefaultAsNamed => {
                                create_default_import_stmt(module_src, ident, runtime_module_ident)
                            }
                            ModuleType::Named => create_named_import_stmt(
                                module_src,
                                ident,
                                runtime_module_ident,
                                imported,
                            ),
                            ModuleType::NamespaceOrAll => create_namespace_import_stmt(
                                module_src,
                                ident,
                                runtime_module_ident,
                            ),
                        }
                        .into(),
                    )
                }
            },
        );

        stmts
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

        if exports.len() > 0 {
            let mut args = vec![
                self.module_id.as_str().as_arg(),
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
        }

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

        if self.runtime_module {
            for (index, registered) in self.resolver.registered_idents.iter().enumerate() {
                module.body.insert(
                    index,
                    decl_var_and_assign_stmt(
                        registered.1,
                        if self.resolver.is_external(registered.0) {
                            external_module_from_global(registered.0)
                        } else {
                            import_module_from_global(registered.0)
                        },
                    )
                    .into(),
                );
            }
        }

        if esm_collector.exports.is_empty() {
            module.visit_mut_with(&mut CommonJsTransformer::new(
                &self.resolver,
                self.module_id.clone(),
                self.runtime_module,
            ));
        }
    }
}

pub fn global_module(
    module_id: String,
    runtime_module: bool,
    external_pattern: Option<String>,
    module_ids: Option<HashMap<String, String>>,
) -> impl VisitMut + Fold {
    as_folder(GlobalModuleTransformer::new(
        module_id,
        runtime_module,
        external_pattern,
        module_ids,
    ))
}
