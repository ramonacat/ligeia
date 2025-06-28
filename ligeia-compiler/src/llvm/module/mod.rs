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
// TODO this should have some other name, this is the real FunctionDeclaration, perhaps the other
// should be renamed?
pub struct FunctionId {
    module_id: ModuleId,
    name: GlobalSymbol,
    r#type: Function,
    visibility: Visibility,
}

impl FunctionId {
    pub(in crate::llvm) const fn name(&self) -> GlobalSymbol {
        self.name
    }
}
