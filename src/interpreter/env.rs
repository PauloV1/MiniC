use std::collections::HashMap;

use crate::ir::ast::CheckedFunDecl;

use super::value::Value;

/// Runtime environment: variable bindings and function registry.
pub struct RuntimeEnv {
    vars: HashMap<String, Value>,
    fns: HashMap<String, CheckedFunDecl>,
}

impl RuntimeEnv {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            fns: HashMap::new(),
        }
    }

    // --- Variable operations ---

    pub fn declare_var(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    pub fn set_var(&mut self, name: &str, value: Value) -> bool {
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), value);
            true
        } else {
            false
        }
    }

    // --- Scope management ---

    /// Full snapshot: used by function calls (complete isolation of local scope).
    pub fn snapshot(&self) -> HashMap<String, Value> {
        self.vars.clone()
    }

    /// Full restore: used by function calls to undo all bindings introduced inside.
    pub fn restore(&mut self, snapshot: HashMap<String, Value>) {
        self.vars = snapshot;
    }

    /// Return the set of currently declared variable names.
    /// Used by blocks to track which variables were in scope before the block.
    pub fn var_names(&self) -> std::collections::HashSet<String> {
        self.vars.keys().cloned().collect()
    }

    /// Remove any variables that were not present in `outer_keys`.
    /// Used on block exit to clean up block-local declarations while keeping
    /// assignments to pre-existing variables.
    pub fn remove_new_vars(&mut self, outer_keys: &std::collections::HashSet<String>) {
        self.vars.retain(|k, _| outer_keys.contains(k));
    }

    // --- Function operations ---

    pub fn register_fn(&mut self, name: String, decl: CheckedFunDecl) {
        self.fns.insert(name, decl);
    }

    pub fn get_fn(&self, name: &str) -> Option<&CheckedFunDecl> {
        self.fns.get(name)
    }
}

impl Default for RuntimeEnv {
    fn default() -> Self {
        Self::new()
    }
}
