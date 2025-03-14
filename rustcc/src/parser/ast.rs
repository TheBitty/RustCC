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

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Type {
    Int,
    Char,
    Void,
    // Add additional primitive types
    Float,
    Double,
    Short,
    Long,
    LongLong,
    UnsignedInt,
    UnsignedChar,
    UnsignedShort,
    UnsignedLong,
    UnsignedLongLong,
    Bool, // For C99 _Bool or C++ bool
    // Keep existing complex types
    Pointer(Box<Type>),
    Array(Box<Type>, Option<usize>),
    Struct(String),
    // Enhanced type qualifiers as separate items
    Const(Box<Type>),
    Volatile(Box<Type>),
    Restrict(Box<Type>), // C99 restrict qualifier
    // Add Union type
    Union(String),
    // Enhanced function type
    Function {
        return_type: Box<Type>,
        parameters: Vec<(String, Type)>,
        is_variadic: bool,
    },
    // TypeDef name (to be resolved during semantic analysis)
    TypeDef(String),
}

impl Type {
    /// Returns the size of this type in bytes
    pub fn size(&self) -> usize {
        match self {
            Type::Void => 0,
            Type::Bool => 1,
            Type::Char | Type::UnsignedChar => 1,
            Type::Short | Type::UnsignedShort => 2,
            Type::Int | Type::UnsignedInt => 4,
            Type::Long | Type::UnsignedLong => 8,
            Type::LongLong | Type::UnsignedLongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::Pointer(_) => 8, // Assume 64-bit pointers
            Type::Array(elem_type, Some(size)) => elem_type.size() * size,
            Type::Array(elem_type, None) => elem_type.size(), // Size unknown, just report element size
            Type::Struct(_) => 0, // Need to look up in symbol table
            Type::Union(_) => 0,  // Need to look up in symbol table
            Type::Const(inner) => inner.size(),
            Type::Volatile(inner) => inner.size(),
            Type::Restrict(inner) => inner.size(),
            Type::Function { .. } => 8, // Function pointers are 8 bytes
            Type::TypeDef(_) => 0, // Need to resolve
        }
    }

    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        // Remove qualifiers for compatibility check
        let self_base = self.remove_qualifiers();
        let other_base = other.remove_qualifiers();
        
        match (self_base, other_base) {
            // Same types are compatible
            (s, o) if std::mem::discriminant(s) == std::mem::discriminant(o) => true,
            
            // Integer types are compatible with each other
            (Type::Int, Type::Long) | (Type::Long, Type::Int) => true,
            (Type::UnsignedInt, Type::UnsignedLong) | (Type::UnsignedLong, Type::UnsignedInt) => true,
            
            // Pointers to compatible types are compatible
            (Type::Pointer(s_inner), Type::Pointer(o_inner)) => 
                s_inner.is_compatible_with(o_inner),
                
            // Arrays of compatible types are compatible
            (Type::Array(s_inner, _), Type::Array(o_inner, _)) => 
                s_inner.is_compatible_with(o_inner),
                
            // Other types are not compatible
            _ => false,
        }
    }
    
    /// Remove qualifiers from a type
    fn remove_qualifiers(&self) -> &Type {
        match self {
            Type::Const(inner) => inner.remove_qualifiers(),
            Type::Volatile(inner) => inner.remove_qualifiers(),
            Type::Restrict(inner) => inner.remove_qualifiers(),
            _ => self,
        }
    }
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
    pub includes: Vec<String>,   // List of include directives for C code
    pub globals: Vec<Statement>, // Global variable declarations
}
