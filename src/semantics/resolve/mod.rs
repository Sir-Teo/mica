mod collector;
mod data;
mod resolver;
mod scope;

pub use data::*;

use crate::syntax::ast::Module;

use collector::ModuleSymbols;
use resolver::Resolver;

pub fn resolve_module(module: &Module) -> Resolved {
    let ModuleSymbols {
        module_scope,
        resolved,
    } = collector::collect_module(module);

    let mut resolver = Resolver::new(module, module_scope, resolved);
    resolver.resolve();
    resolver.into_resolved()
}
