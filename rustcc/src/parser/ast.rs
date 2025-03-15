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
    FloatLiteral(f64),  // Add support for floating-point literals
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
    SizeOfType(Type),  // sizeof(type)
    AlignOf(Type),     // _Alignof(type) - C11
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
    CompoundLiteral {  // (Type){initializers} - C99
        type_name: Type,
        initializers: Vec<Expression>,
    },
    GenericSelection {  // _Generic(expr, type1: expr1, type2: expr2, ...) - C11
        controlling_expr: Box<Expression>,
        associations: Vec<(Type, Expression)>,
        default_expr: Option<Box<Expression>>,
    },
    StaticAssert {  // _Static_assert(expr, message) - C11
        condition: Box<Expression>,
        message: String,
    },
    AtomicExpr {  // Atomic expressions - C11
        operation: AtomicOp,
        operands: Vec<Expression>,
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
        alignment: Option<usize>,  // _Alignas specifier - C11
    },
    ArrayDeclaration {
        name: String,
        data_type: Option<Type>,
        size: Option<Expression>,  // None for VLAs determined at runtime
        initializer: Expression,
        is_global: bool,
        alignment: Option<usize>,  // _Alignas specifier - C11
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
    Goto(String),  // Goto statement
    Label(String, Box<Statement>),  // Label statement
    StaticAssert {  // _Static_assert declaration - C11
        condition: Expression,
        message: String,
    },
    AtomicBlock(Vec<Statement>),  // Atomic compound statement - C11
    ThreadLocal {  // _Thread_local declaration - C11
        declaration: Box<Statement>,
    },
    NoReturn {  // _Noreturn function - C11
        declaration: Box<Statement>,
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
    // C99/C11 specific types
    Complex,    // _Complex
    Imaginary,  // _Imaginary
    Atomic(Box<Type>),  // _Atomic type - C11
    // C11 Generic selections
    Generic {
        controlling_type: Box<Type>,
        associations: Vec<(Type, Type)>,
        default_type: Option<Box<Type>>,
    },
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
            Type::Complex => 8, // _Complex type size
            Type::Imaginary => 8, // _Imaginary type size
            Type::Atomic(_) => 8, // _Atomic type size
            Type::Generic { .. } => 8, // Generic type size
        }
    }

    /// Check if this type is compatible with another type
    #[allow(dead_code)]
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        // Remove qualifiers for compatibility check
        let self_base = self.remove_qualifiers();
        let other_base = other.remove_qualifiers();
        
        match (self_base, other_base) {
            // Same types are compatible
            (s, o) if std::mem::discriminant(s) == std::mem::discriminant(o) => true,
            
            // Integer types are compatible with each other (with potential truncation/sign extension)
            (Type::Int, Type::Short) | (Type::Short, Type::Int) => true,
            (Type::Int, Type::Long) | (Type::Long, Type::Int) => true,
            (Type::Int, Type::LongLong) | (Type::LongLong, Type::Int) => true,
            (Type::Short, Type::Long) | (Type::Long, Type::Short) => true,
            (Type::Short, Type::LongLong) | (Type::LongLong, Type::Short) => true,
            (Type::Long, Type::LongLong) | (Type::LongLong, Type::Long) => true,
            
            // Unsigned integer types are compatible with each other
            (Type::UnsignedInt, Type::UnsignedShort) | (Type::UnsignedShort, Type::UnsignedInt) => true,
            (Type::UnsignedInt, Type::UnsignedLong) | (Type::UnsignedLong, Type::UnsignedInt) => true,
            (Type::UnsignedInt, Type::UnsignedLongLong) | (Type::UnsignedLongLong, Type::UnsignedInt) => true,
            (Type::UnsignedShort, Type::UnsignedLong) | (Type::UnsignedLong, Type::UnsignedShort) => true,
            (Type::UnsignedShort, Type::UnsignedLongLong) | (Type::UnsignedLongLong, Type::UnsignedShort) => true,
            (Type::UnsignedLong, Type::UnsignedLongLong) | (Type::UnsignedLongLong, Type::UnsignedLong) => true,
            
            // Signed and unsigned integer types are compatible (with sign extension/truncation)
            (Type::Int, Type::UnsignedInt) | (Type::UnsignedInt, Type::Int) => true,
            (Type::Short, Type::UnsignedShort) | (Type::UnsignedShort, Type::Short) => true,
            (Type::Long, Type::UnsignedLong) | (Type::UnsignedLong, Type::Long) => true,
            (Type::LongLong, Type::UnsignedLongLong) | (Type::UnsignedLongLong, Type::LongLong) => true,
            
            // Char types are compatible with integer types
            (Type::Char, Type::Int) | (Type::Int, Type::Char) => true,
            (Type::Char, Type::UnsignedInt) | (Type::UnsignedInt, Type::Char) => true,
            (Type::UnsignedChar, Type::Int) | (Type::Int, Type::UnsignedChar) => true,
            (Type::UnsignedChar, Type::UnsignedInt) | (Type::UnsignedInt, Type::UnsignedChar) => true,
            
            // Bool is compatible with integer types
            (Type::Bool, Type::Int) | (Type::Int, Type::Bool) => true,
            (Type::Bool, Type::UnsignedInt) | (Type::UnsignedInt, Type::Bool) => true,
            
            // Float and double are compatible with each other
            (Type::Float, Type::Double) | (Type::Double, Type::Float) => true,
            
            // Integer types can be converted to floating point types
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
            (Type::Int, Type::Double) | (Type::Double, Type::Int) => true,
            (Type::UnsignedInt, Type::Float) | (Type::Float, Type::UnsignedInt) => true,
            (Type::UnsignedInt, Type::Double) | (Type::Double, Type::UnsignedInt) => true,
            
            // Pointers to compatible types are compatible
            (Type::Pointer(s_inner), Type::Pointer(o_inner)) => 
                s_inner.is_compatible_with(o_inner) || matches!(**s_inner, Type::Void) || matches!(**o_inner, Type::Void),
                
            // Arrays of compatible types are compatible
            (Type::Array(s_inner, _), Type::Array(o_inner, _)) => 
                s_inner.is_compatible_with(o_inner),
                
            // Arrays can decay to pointers
            (Type::Array(s_inner, _), Type::Pointer(o_inner)) => 
                s_inner.is_compatible_with(o_inner),
            (Type::Pointer(s_inner), Type::Array(o_inner, _)) => 
                s_inner.is_compatible_with(o_inner),
            
            // Function types are compatible if return types and parameter types are compatible
            (Type::Function { return_type: s_ret, parameters: s_params, is_variadic: s_var },
             Type::Function { return_type: o_ret, parameters: o_params, is_variadic: o_var }) => {
                // Return types must be compatible
                if !s_ret.is_compatible_with(o_ret) {
                    return false;
                }
                
                // Variadic functions are only compatible with other variadic functions
                if s_var != o_var {
                    return false;
                }
                
                // Parameter counts must match
                if s_params.len() != o_params.len() {
                    return false;
                }
                
                // Each parameter type must be compatible
                for ((_, s_type), (_, o_type)) in s_params.iter().zip(o_params.iter()) {
                    if !s_type.is_compatible_with(o_type) {
                        return false;
                    }
                }
                
                true
            }
            
            // Other types are not compatible
            _ => false,
        }
    }
    
    /// Remove qualifiers from a type
    #[allow(dead_code)]
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

// C11 Atomic operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AtomicOp {
    Load,
    Store,
    Exchange,
    CompareExchange,
    FetchAdd,
    FetchSub,
    FetchAnd,
    FetchOr,
    FetchXor,
}
