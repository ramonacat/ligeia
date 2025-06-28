pub mod builder;

use super::module::built::Module;

pub struct Package {
    module: Module,
}

impl Package {
    pub const fn new(module: Module) -> Self {
        Self { module }
    }

    pub(crate) fn into_module(self) -> Module {
        self.module
    }
}
