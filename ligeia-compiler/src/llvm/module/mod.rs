pub mod builder;
pub mod built;

use builder::ModuleBuilder;
use built::Module;

use super::{global_symbol::GlobalSymbol, types::function::Function};

pub(in crate::llvm) trait AnyModule {}

impl AnyModule for ModuleBuilder<'_> {}
impl AnyModule for Module {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// TODO This should also contain a reference to the package id? or GlobalSymbol should have a
// reference to the specific GlobalSymbols? So that in case of building multiple packages, a mixup
// will just result in an obvious error
pub struct ModuleId(GlobalSymbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId {
    module_id: ModuleId,
    name: GlobalSymbol,
    r#type: Function,
}

impl FunctionId {
    pub(in crate::llvm) const fn name(&self) -> GlobalSymbol {
        self.name
    }
}
