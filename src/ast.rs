use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<AttributeArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeArg {
    Identifier(String),
    String(String),
    Integer(i64),
    Boolean(bool),
    KeyValue(String, Box<AttributeArg>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Program {
    Statements(Vec<Statement>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
    Assignment(String, Expression),
    MethodDef(MethodDef),
    SystemDef(SystemDef),
    MessageDef(MessageDef),
    ModuleDef(ModuleDef),
    Require(String),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Return(Option<Expression>),
    Break,
    Next,
    Attributes(Vec<Attribute>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemDef {
    pub name: String,
    pub body: Vec<Statement>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDef {
    pub name: String,
    pub body: Vec<Statement>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Vec<Statement>,
    pub elsif_blocks: Vec<(Expression, Vec<Statement>)>,
    pub else_block: Option<Vec<Statement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStatement {
    pub variable: String,
    pub iterable: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    String(String),
    StringInterpolation(Vec<InterpolationPart>), // String with embedded expressions
    Symbol(String),
    NamespaceAccess(Vec<String>),
    Boolean(bool),
    Nil,
    Identifier(String),
    Array(Vec<Expression>),
    Hash(Vec<(Expression, Expression)>),
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        exclusive: bool,
    },
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Call {
        receiver: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    Index {
        expr: Box<Expression>,
        index: Box<Expression>,
    },
    Lambda {
        params: Vec<String>,
        body: Vec<Statement>,
    },
    Block(Vec<Statement>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Literal(String),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,

    // Comparison
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Spaceship, // <=>

    // Logical
    And,
    Or,

    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
    BitwiseNot,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Integer(n) => write!(f, "{}", n),
            Expression::Float(n) => write!(f, "{}", n),
            Expression::String(s) => write!(f, "\"{}\"", s),
            Expression::StringInterpolation(parts) => {
                write!(f, "\"")?;
                for part in parts {
                    match part {
                        InterpolationPart::Literal(s) => write!(f, "{}", s)?,
                        InterpolationPart::Expression(e) => write!(f, "#{{{}}}", e)?,
                    }
                }
                write!(f, "\"")
            }
            Expression::Symbol(s) => write!(f, ":{}", s),
            Expression::Boolean(b) => write!(f, "{}", b),
            Expression::Nil => write!(f, "nil"),
            Expression::Identifier(name) => write!(f, "{}", name),
            Expression::Array(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Expression::Hash(pairs) => {
                write!(f, "{{")?;
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} => {}", key, value)?;
                }
                write!(f, "}}")
            }
            Expression::Range {
                start,
                end,
                exclusive,
            } => {
                if *exclusive {
                    write!(f, "{}...{}", start, end)
                } else {
                    write!(f, "{}..{}", start, end)
                }
            }
            Expression::Binary(op, left, right) => {
                write!(f, "({} {:?} {})", left, op, right)
            }
            Expression::Unary(op, expr) => {
                write!(f, "({:?} {})", op, expr)
            }
            Expression::Call {
                receiver,
                method,
                args,
            } => {
                write!(f, "{}.{}(", receiver, method)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expression::Index { expr, index } => {
                write!(f, "{}[{}]", expr, index)
            }
            Expression::Lambda { params, .. } => {
                write!(f, "lambda {{ |{}| ... }}", params.join(", "))
            }
            Expression::Block(_) => write!(f, "{{ ... }}"),
            Expression::NamespaceAccess(parts) => {
                write!(f, "{}", parts.join("::"))
            }
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Power => "**",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::LessThan => "<",
            BinaryOp::LessThanOrEqual => "<=",
            BinaryOp::GreaterThan => ">",
            BinaryOp::GreaterThanOrEqual => ">=",
            BinaryOp::Spaceship => "<=>",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitwiseAnd => "&",
            BinaryOp::BitwiseOr => "|",
            BinaryOp::BitwiseXor => "^",
            BinaryOp::LeftShift => "<<",
            BinaryOp::RightShift => ">>",
        };
        write!(f, "{}", symbol)
    }
}
