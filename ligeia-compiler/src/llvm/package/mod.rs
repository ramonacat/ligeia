use super::{global_symbol::GlobalSymbols, module::Module};

pub struct PackageBuilder {
    global_symbols: GlobalSymbols,
}

impl PackageBuilder {
    pub fn new() -> Self {
        Self {
            global_symbols: GlobalSymbols::new(),
        }
    }

    pub fn add_module(&mut self, name: &str) -> Module {
        Module::new(&mut self.global_symbols, name)
    }
}
