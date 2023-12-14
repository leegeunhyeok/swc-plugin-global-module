mod cjs_transformer;
mod constants;
mod esm_collector;
mod helpers;
mod module_mapper;

use cjs_transformer::CommonJsTransformer;
use constants::{ESM_API_NAME, GLOBAL, MODULE, MODULE_EXTERNAL_NAME};
use esm_collector::{EsModuleCollector, ExportModule, ImportModule, ModuleType};
use helpers::{
    create_default_import_stmt, create_named_import_stmt, create_namespace_import_stmt,
    decl_var_and_assign_stmt, external_module_from_global, import_module_from_global, obj_lit,
    obj_member_expr,
};
use module_mapper::ModuleMapper;
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
    module_name: String,
    commonjs: bool,
    runtime_module: bool,
    external: HashMap<String, bool>,
    module_mapper: ModuleMapper,
}

impl GlobalModuleTransformer {
    fn new(
        module_name: String,
        commonjs: bool,
        runtime_module: bool,
        external: Option<Vec<String>>,
        import_paths: Option<HashMap<String, String>>,
    ) -> Self {
        GlobalModuleTransformer {
            module_name,
            commonjs,
            runtime_module,
            external: external
                .and_then(|external| {
                    Some(
                        external
                            .iter()
                            .map(|external| (external.clone(), false))
                            .collect::<HashMap<String, bool>>(),
                    )
                })
                .unwrap_or_default(),
            module_mapper: ModuleMapper::new(import_paths),
        }
    }

    fn is_external(&self, src: &String) -> bool {
        self.external.contains_key(src)
    }

    fn register_external_module(&mut self, stmts: &mut Vec<ModuleItem>, src: &String) -> bool {
        if let Some(registered) = self.external.get(src) {
            if *registered {
                return true;
            }

            self.external.insert(src.clone(), true);
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
            return true;
        }
        false
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
                    let runtime_module_ident: Option<&Ident> = if self.runtime_module {
                        Some(
                            self.module_mapper
                                .get_ident_by_src(module_src, self.is_external(module_src)),
                        )
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
            for (index, registered) in self.module_mapper.registered_idents.iter().enumerate() {
                module.body.insert(
                    index,
                    decl_var_and_assign_stmt(
                        registered.1,
                        if self.is_external(registered.0) {
                            external_module_from_global(registered.0)
                        } else {
                            import_module_from_global(registered.0)
                        },
                    )
                    .into(),
                );
            }
        }

        if self.commonjs && esm_collector.exports.is_empty() {
            module.visit_mut_with(&mut CommonJsTransformer::new(
                &self.module_mapper,
                self.module_name.clone(),
                self.runtime_module,
            ));
        }
    }
}

pub fn global_module(
    module_name: String,
    commonjs: bool,
    runtime_module: bool,
    external: Option<Vec<String>>,
    import_paths: Option<HashMap<String, String>>,
) -> impl VisitMut + Fold {
    as_folder(GlobalModuleTransformer::new(
        module_name,
        commonjs,
        runtime_module,
        external,
        import_paths,
    ))
}
