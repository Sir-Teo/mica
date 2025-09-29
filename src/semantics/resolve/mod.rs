mod collector;
mod data;
mod resolver;
mod scope;
mod workspace;

pub use data::*;

use crate::syntax::ast::Module;

use collector::ModuleSymbols;
use resolver::Resolver;
use workspace::ModuleGraph;

use std::collections::HashMap;

pub fn resolve_module(module: &Module) -> Resolved {
    let mut graph = ModuleGraph::new();
    let ModuleSymbols {
        module_scope,
        resolved,
        ..
    } = graph.add_module(module);

    let mut resolver = Resolver::new(module, module_scope, resolved, &graph);
    resolver.resolve();
    resolver.into_resolved()
}

pub fn resolve_modules<'a, I>(modules: I) -> HashMap<Vec<String>, Resolved>
where
    I: IntoIterator<Item = &'a Module>,
{
    let modules: Vec<&'a Module> = modules.into_iter().collect();
    let mut graph = ModuleGraph::new();
    let mut collected = Vec::new();

    for module in modules.iter().copied() {
        let symbols = graph.add_module(module);
        collected.push((module.name.clone(), symbols));
    }

    let mut results = HashMap::new();
    for (module_path, symbols) in collected {
        let module = modules
            .iter()
            .copied()
            .find(|m| m.name == module_path)
            .expect("module present in workspace");
        let ModuleSymbols {
            module_scope,
            resolved,
            ..
        } = symbols;
        let mut resolver = Resolver::new(module, module_scope, resolved, &graph);
        resolver.resolve();
        results.insert(module_path, resolver.into_resolved());
    }

    results
}
