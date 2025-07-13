use crate::{Visibility, types};

pub struct FunctionSignature {
    name: String,
    r#type: types::Function,
    visibility: Visibility,
}

impl FunctionSignature {
    pub fn new(name: impl Into<String>, r#type: types::Function, visibility: Visibility) -> Self {
        Self {
            name: name.into(),
            r#type,
            visibility,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) const fn r#type(&self) -> types::Function {
        self.r#type
    }

    pub(crate) const fn visibility(&self) -> Visibility {
        self.visibility
    }
}
