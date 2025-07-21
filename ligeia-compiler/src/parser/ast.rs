#![allow(unused, dead_code)]

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Identifier(pub String);

#[derive(Debug)]
pub enum Type {
    Unit,
    U64,
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
pub struct SourceFile {
    pub declarations: Vec<Declaration>,
    pub name: String,
}
