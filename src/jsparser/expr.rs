use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Empty,                   //base
    Unary(Unary, Box<Expr>), // !a  !~+-a
    Unexpected(String),      //异常
    Identifier(String),
    Template(Vec<String>, Vec<Expr>), //``模板
    Literal(String),
    Call(Box<Expr>, Vec<Expr>),            // a()  a.b()
    Member(Box<Expr>, Box<Expr>),          //a.b a[b]
    Sequence(Vec<Expr>),                   // a[1,2,3,4]
    Infix(Box<Expr>, Operator, Box<Expr>), //算术符号 a+b  +-*/   a && b  逻辑符号 &&,||,!
    Update(Box<Expr>, String, bool),       //a++/++a     bool:存放++的前后位置
    Variable(Vec<(Variable, String, Expr)>),
    Assignment(Vec<(String, Expr)>),
    If(Box<Expr>, Box<Expr>, Box<Expr>), //if
    Block(Vec<Expr>),
    Expression(Box<Expr>),
    Return(Box<Expr>),
    For(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>), //for
    ForIn(Box<Expr>, Box<Expr>),                     //for in
    ForOf(Box<Expr>, Box<Expr>),                     //for of
    Break,
    Continue,
    Function(Box<Expr>, Vec<Expr>, Box<Expr>), //function
    While(Box<Expr>, Box<Expr>),
    DoWhile(Box<Expr>, Box<Expr>), //存放顺序与while一致
    Switch(Box<Expr>, Vec<Expr>),
    SwitchCase(Box<Expr>, Vec<Expr>),
    Object(HashMap<String, Expr>), //json
    Ref(String),                   //let a=1; let b ={a};//b.a=a=1;
    Array(Vec<Expr>),              //array
}

impl Expr {
    pub fn to_raw(&self) -> String {
        match self {
            Expr::Identifier(t) => t.to_string(),
            Expr::Literal(t) => t.to_string(),
            _ => format!("<{:?}>", self),
        }
    }
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

    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    // In,
    // Of,
}
