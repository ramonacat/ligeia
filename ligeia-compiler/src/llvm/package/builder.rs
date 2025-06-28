use super::Package;
use crate::llvm::{global_symbol::GlobalSymbols, module::builder::ModuleBuilder};

pub struct PackageBuilder<'symbols> {
    global_symbols: &'symbols GlobalSymbols,
    modules: Vec<ModuleBuilder<'symbols>>,
}

impl<'package, 'symbols> PackageBuilder<'symbols>
where
    'symbols: 'package,
{
    pub const fn new(global_symbols: &'symbols GlobalSymbols) -> Self {
        Self {
            global_symbols,
            modules: vec![],
        }
    }

    pub fn add_module(&'package mut self, name: &str) -> &'package mut ModuleBuilder<'symbols> {
        self.modules
            .push(ModuleBuilder::new(self.global_symbols, name));

        self.modules.last_mut().unwrap()
    }

    pub(crate) fn build(self) -> Package {
        let mut built_modules = self
            .modules
            .into_iter()
            .map(ModuleBuilder::build)
            .collect::<Vec<_>>();

        let final_module = built_modules
            .pop()
            .expect("package should contain at least a single module");

        for module in built_modules {
            final_module.link(module);
        }

        Package::new(final_module)
    }
}
