#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Empty, //base
    Unary(Unary, Box<Expr>),
    Unexpected(String), //异常
    Identifier(String),

    TemplateLiteral(Box<Expr>, Box<Expr>), //``
    Literal(String,String),
    Assignment(String, Box<Expr>), // a=b
    // Prefix(Prefix, Box<Expr>),     // !a  -1
    Call(Box<Expr>, Vec<Expr>), // a()  a.b()

    Member(Box<Expr>, Box<Expr>), //a.b a[b]
    Sequence(Vec<Expr>),          // a[1,2,3,4]

    Infix(Box<Expr>, Operator, Box<Expr>), //算术符号 a+b  +-*/   a && b  逻辑符号 &&,||,!
    Update(Box<Expr>, String, bool),       //a++/++a     bool:存放++的前后位置
    Variable(Variable, String, Box<Expr>), //let a =
    If(Box<Expr>, Box<Expr>, Box<Expr>),   //if
    BlockStatement(Vec<Expr>),
    Expression(Box<Expr>),
    Return(Box<Expr>),
    For(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>), //for
    ForIn(Box<Expr>, Box<Expr>),                     //for in
    ForOf(Box<Expr>, Box<Expr>),                     //for of
    Break,
    Continue,
    Function(Box<Expr>, Vec<Expr>, Box<Expr>), //function
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Var,
    Let,
    Const,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Unary {
    /// !
    Not,
    /// +
    Plus,
    /// -
    Minus,
    /// ~
    BitNot,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Prefix {
    Negate, // -expr
    Abs,    // +a
    Not,    // !
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Plus,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Or,
    And,
    Not,
    LShift,
    RShift,
    Equal,
    NE,
    GT,
    GTE,
    LT,
    LTE,
    BitOr,
    BitXor,
    BitAnd,
    INC,
    DEC,
    // In,
    // Of,
}
