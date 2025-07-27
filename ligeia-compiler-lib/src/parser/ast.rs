#![allow(unused, dead_code)]

use std::fmt::Display;

// TODO: intern the IDs, and make this Copy
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Unit,
    U64,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::U64 => write!(f, "u64"),
        }
    }
}

#[derive(Debug)]
pub enum Literal {
    // TODO support u128???
    UnsignedInteger(u64),
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    VariableReference(Identifier),

    Sum(Box<Expression>, Box<Expression>),
}

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Return(Expression),
}

#[derive(Debug)]
pub struct Argument {
    pub name: Identifier,
    pub r#type: Type,
}

#[derive(Debug)]
pub enum FunctionBody {
    Extern(Identifier),
    Statements(Vec<Statement>),
}

#[derive(Debug)]
pub struct Function {
    pub name: Identifier,
    pub arguments: Vec<Argument>,
    pub return_type: Type,
    pub body: FunctionBody,
}

#[derive(Debug)]
pub enum Declaration {
    Function(Function),
}

#[derive(Debug)]
#[must_use]
pub struct SourceFile {
    pub declarations: Vec<Declaration>,
    pub name: String,
}
