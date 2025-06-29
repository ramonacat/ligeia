pub mod builder;
pub mod built;

use builder::ModuleBuilder;
use built::Module;

use super::{
    function::declaration::Visibility, global_symbol::GlobalSymbol, package::id::PackageId,
    types::function::Function,
};

pub(in crate::llvm) trait AnyModule {}

impl AnyModule for ModuleBuilder {}
impl AnyModule for Module {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(PackageId, GlobalSymbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionDeclaration {
    module_id: ModuleId,
    name: GlobalSymbol,
    r#type: Function,
    visibility: Visibility,
}

impl FunctionDeclaration {
    pub(in crate::llvm) const fn name(&self) -> GlobalSymbol {
        self.name
    }
}
