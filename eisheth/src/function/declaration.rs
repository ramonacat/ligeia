use crate::types;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Internal,
    Export,
}

pub struct FunctionDeclarationDescriptor {
    name: String,
    r#type: types::Function,
    visibility: Visibility,
}

impl FunctionDeclarationDescriptor {
    pub fn new(name: impl Into<String>, r#type: types::Function, visibility: Visibility) -> Self {
        Self {
            // TODO Should we convert into CString on creation?
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
