use std::collections::HashMap;

use serde::Deserialize;
use swc_core::{
    ecma::{ast::Program, visit::FoldWith},
    plugin::{
        metadata::TransformPluginMetadataContextKind, plugin_transform,
        proxies::TransformPluginProgramMetadata,
    },
};
use swc_global_module::global_module;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GlobalModuleOptions {
    runtime_module: Option<bool>,
    external_pattern: Option<String>,
    import_paths: Option<HashMap<String, String>>,
}

#[plugin_transform]
pub fn global_module_plugin(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<GlobalModuleOptions>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for swc-plugin-global-module"),
    )
    .expect("invalid config for swc-plugin-global-module");

    program.fold_with(&mut global_module(
        metadata
            .get_context(&TransformPluginMetadataContextKind::Filename)
            .unwrap_or_default(),
        config.runtime_module.unwrap_or(false),
        config.external_pattern,
        config.import_paths,
    ))
}
