use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use swc_core::ecma::{ast::*, utils::private_ident};

pub type ImportPaths = HashMap<String, String>;

pub struct ModuleResolver {
    pub registered_idents: BTreeMap<String, Ident>,
    external_regex: Option<Regex>,
    import_paths: Option<ImportPaths>,
    normalize_regex: Regex,
}

impl ModuleResolver {
    pub fn new(external_pattern: Option<String>, import_paths: Option<ImportPaths>) -> Self {
        ModuleResolver {
            external_regex: external_pattern
                .and_then(|pattern| Some(Regex::new(pattern.as_str()).unwrap())),
            import_paths,
            registered_idents: BTreeMap::new(),
            normalize_regex: Regex::new(r"[^a-zA-Z0-9]").unwrap(),
        }
    }

    pub fn get_ident_by_src(&mut self, src: &String, is_external: bool) -> &Ident {
        let module_path = self
            .to_actual_path(src, is_external)
            .unwrap_or(src.to_string());
        self.registered_idents
            .entry(module_path)
            .or_insert(private_ident!(self
                .normalize_regex
                .replace_all(format!("_{src}").as_str(), "_")
                .to_string()))
    }

    pub fn to_actual_path(&self, src: &String, is_external: bool) -> Option<String> {
        if is_external {
            None
        } else if let Some(actual_path) = self
            .import_paths
            .as_ref()
            .and_then(|import_paths| import_paths.get(src))
        {
            Some(actual_path.clone())
        } else {
            None
        }
    }

    pub fn is_external(&self, src: &String) -> bool {
        if let Some(regex) = &self.external_regex {
            regex.is_match(src.as_str())
        } else {
            false
        }
    }
}
