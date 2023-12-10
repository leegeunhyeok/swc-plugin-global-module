use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use swc_core::ecma::{ast::*, utils::private_ident};

pub type ImportPaths = HashMap<String, String>;

pub struct ModuleMapper {
    pub registered_idents: BTreeMap<String, Ident>,
    import_paths: Option<ImportPaths>,
    normalize_regex: Regex,
}

impl ModuleMapper {
    pub fn new(import_paths: Option<ImportPaths>) -> Self {
        ModuleMapper {
            import_paths,
            registered_idents: BTreeMap::new(),
            normalize_regex: Regex::new(r"[^a-zA-Z0-9]").unwrap(),
        }
    }

    pub fn register_ident_by_src(&mut self, src: &String) -> &Ident {
        let module_path = self.to_actual_path(src).unwrap_or(src.to_string());
        self.registered_idents
            .entry(module_path)
            .or_insert(private_ident!(self
                .normalize_regex
                .replace_all(format!("_{src}").as_str(), "_")
                .to_string()))
    }

    fn to_actual_path(&self, src: &String) -> Option<String> {
        if let Some(actual_path) = self
            .import_paths
            .as_ref()
            .and_then(|import_paths| import_paths.get(src))
        {
            return Some(actual_path.clone());
        }
        None
    }
}
