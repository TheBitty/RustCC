#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LogicalAnd,
    LogicalOr,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    LogicalNot,    // for !expr
    Negate,        // for -expr
    PreIncrement,  // for ++expr
    PreDecrement,  // for --expr
    PostIncrement, // for expr++
    PostDecrement, // for expr--
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UnaryOp {
    // Keep these variants for backward compatibility
    Negate,        // -
    LogicalNot,    // !
    BitwiseNot,    // ~
    AddressOf,     // &
    Dereference,   // *
    PreIncrement,  // ++x
    PreDecrement,  // --x
    PostIncrement, // x++
    PostDecrement, // x--
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum OperatorType {
    Binary(BinaryOp),
    Unary(UnaryOp),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expression {
    IntegerLiteral(i32),
    StringLiteral(String),
    CharLiteral(char),
    BinaryOperation {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    UnaryOperation {
        operator: OperatorType,
        operand: Box<Expression>,
    },
    Variable(String),
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    TernaryIf {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    Cast {
        target_type: Type,
        expr: Box<Expression>,
    },
    SizeOf(Box<Expression>),
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    ArrayLiteral(Vec<Expression>),
    StructFieldAccess {
        object: Box<Expression>,
        field: String,
    },
    PointerFieldAccess {
        pointer: Box<Expression>,
        field: String,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Statement {
    Return(Expression),
    VariableDeclaration {
        name: String,
        data_type: Option<Type>,
        initializer: Expression,
        is_global: bool,
    },
    ArrayDeclaration {
        name: String,
        data_type: Option<Type>,
        size: Option<Expression>,
        initializer: Expression,
        is_global: bool,
    },
    #[allow(clippy::enum_variant_names)]
    ExpressionStatement(Expression),
    Block(Vec<Statement>),
    If {
        condition: Expression,
        then_block: Box<Statement>,
        else_block: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    For {
        initializer: Option<Box<Statement>>,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Box<Statement>,
    },
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
    },
    Break,
    Continue,
    Switch {
        expression: Expression,
        cases: Vec<SwitchCase>,
    },
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub value: Option<Expression>, // None for default case
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Type {
    Int,
    Char,
    Void,
    Pointer(Box<Type>),
    Array(Box<Type>, Option<usize>),
    Struct(String),
    Const(Box<Type>),
    Function {
        return_type: Box<Type>,
        parameters: Vec<(String, Type)>,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FunctionParameter {
    pub name: String,
    pub data_type: Type,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StructField {
    pub name: String,
    pub data_type: Type,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Function {
    pub name: String,
    pub return_type: Type,
    pub parameters: Vec<FunctionParameter>,
    pub body: Vec<Statement>,
    pub is_variadic: bool,
    pub is_external: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Program {
    pub functions: Vec<Function>,
    pub structs: Vec<Struct>,
    pub includes: Vec<String>, // List of include directives for C code
    pub globals: Vec<Statement>, // Global variable declarations
}
