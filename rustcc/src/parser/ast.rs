#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
pub enum Expression {
    IntegerLiteral(i32),
    BinaryOperation {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Variable(String),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Return(Expression),
    VariableDeclaration {
        name: String,
        initializer: Expression,
    },
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
} 