use std::{collections::HashMap, path::PathBuf};

use swc_ecma_parser::{Syntax, TsConfig};
use swc_ecma_transforms_testing::test_fixture;
use swc_global_module::global_module;

// ESM
#[testing::fixture("tests/fixture/**/input.js")]
fn fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");
    let runtime = !filename.contains("non-runtime");

    let external = if filename.contains("external") {
        Some(String::from("^(react|react-native)"))
    } else {
        None
    };

    let module_ids = if filename.contains("module_ids") {
        Some(HashMap::from([(
            String::from("react"),
            String::from("12345"),
        )]))
    } else {
        None
    };

    test_fixture(
        Syntax::Typescript(TsConfig {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| {
            global_module(
                String::from("test.js"),
                runtime,
                external.to_owned(),
                module_ids.to_owned(),
            )
        },
        &input,
        &output,
        Default::default(),
    );
}
