use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Empty,                   //base
    Unary(Unary, Box<Expr>), // !a  !~+-a
    Unexpected(String),      //异常
    Identifier(String),

    TemplateLiteral(Box<Expr>, Box<Expr>), //``
    Literal2(String), //保留原始raw    标记为2的表示未确定最终格式，最后需要统一规划
    Literal(String, String),
    Assignment(String, Box<Expr>), // a=b
    Call(Box<Expr>, Vec<Expr>),    // a()  a.b()

    Member(Box<Expr>, Box<Expr>), //a.b a[b]
    Sequence(Vec<Expr>),          // a[1,2,3,4]

    Infix(Box<Expr>, Operator, Box<Expr>), //算术符号 a+b  +-*/   a && b  逻辑符号 &&,||,!
    Update(Box<Expr>, String, bool),       //a++/++a     bool:存放++的前后位置
    Variable(Variable, String, Box<Expr>), //let a =
    Variable2(Vec<(Variable, String, Expr)>),
    Assignment2(Vec<(String, Expr)>),
    If(Box<Expr>, Box<Expr>, Box<Expr>), //if
    BlockStatement(Vec<Expr>),
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

    Object(HashMap<String, Expr>), //json
    Ref(String),                   //let a=1; let b ={a};//b.a=a=1;
    Array(Vec<Expr>),              //array
}

impl Expr {
    pub fn to_raw(&self) -> String {
        match self {
            // Expr::Empty => todo!(),
            // Expr::Unary(unary, expr) => todo!(),
            // Expr::Unexpected(_) => todo!(),
            Expr::Identifier(t) => t.to_string(),
            // Expr::TemplateLiteral(expr, expr1) => todo!(),
            // Expr::Literal2(_) => todo!(),
            Expr::Literal(t, _) => t.to_string(),
            // Expr::Assignment(_, expr) => todo!(),
            // Expr::Call(expr, vec) => todo!(),
            // Expr::Member(expr, expr1) => todo!(),
            // Expr::Sequence(vec) => todo!(),
            // Expr::Infix(expr, operator, expr1) => todo!(),
            // Expr::Update(expr, _, _) => todo!(),
            // Expr::Variable(variable, _, expr) => todo!(),
            // Expr::Variable2(vec) => todo!(),
            // Expr::Assignment2(vec) => todo!(),
            // Expr::If(expr, expr1, expr2) => todo!(),
            // Expr::BlockStatement(vec) => todo!(),
            // Expr::Expression(expr) => todo!(),
            // Expr::Return(expr) => todo!(),
            // Expr::For(expr, expr1, expr2, expr3) => todo!(),
            // Expr::ForIn(expr, expr1) => todo!(),
            // Expr::ForOf(expr, expr1) => todo!(),
            // Expr::Break => todo!(),
            // Expr::Continue => todo!(),
            // Expr::Function(expr, vec, expr1) => todo!(),
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
