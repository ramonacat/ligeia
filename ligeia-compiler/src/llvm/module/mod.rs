pub mod built;
pub mod builder;

use builder::ModuleBuilder;
use built::Module;

use super::{
    global_symbol::GlobalSymbol,
    types::function::Function,
};

pub(in crate::llvm) trait AnyModule {}

impl AnyModule for ModuleBuilder<'_> {}
impl AnyModule for Module {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(GlobalSymbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId {
    module_id: ModuleId,
    name: GlobalSymbol,
    r#type: Function,
}
