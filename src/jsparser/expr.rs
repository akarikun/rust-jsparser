#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Empty,              //base
    Unary(Unary,Box<Expr>),
    Unexpected(String), //异常
    Identifier(String),
    Number(f64),
    Assignment(String, Box<Expr>), // a=b
    Prefix(Prefix, Box<Expr>),  // !a  -1
    Call(Box<Expr>, Vec<Expr>), // a()  a.b()

    Member(Box<Expr>, Box<Expr>),             //a.b a[b]
    Sequence(Vec<Expr>),                      // a[1,2,3,4]

    Infix(Box<Expr>, Operator, Box<Expr>), //算术符号 a+b  +-*/   a && b  逻辑符号 &&,||,!
    Update(Box<Expr>, String, bool),     //a++/++a     bool:存放++的前后位置
    Variable(Variable, String, Box<Expr>),   //let a =
    If(Box<Expr>, Box<Expr>, Box<Expr>),   //if
    BlockStatement(Vec<Expr>),
    Expression(Box<Expr>),
    Return(Box<Expr>),
    For(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>), //for
    ForIn(Box<Expr>, Box<Expr>),                     //for in
    ForOf(Box<Expr>, Box<Expr>),                     //for of
}
#[derive(Debug, Clone,PartialEq)]
pub enum Variable{
    Var,
    Let,
    Const,
}

#[derive(Debug, Clone,PartialEq)]
pub enum Unary{
    /// !
    Not,
    /// +
    Plus,
    /// -
    Minus,
    /// ~
    BitNot,
}
#[derive(Debug, Clone,PartialEq)]
pub enum Prefix {
    Negate, // -expr
    Abs,    // +a
    Not,    // !
}

#[derive(Debug, Clone,PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
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

    In,
    Of,
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Expr>,
}

impl Expr {
    pub fn calc(&self) -> Option<f64> {
        match &self {
            Expr::Number(val) => return Some(*val),
            Expr::Infix(left, op, right) => {
                let left_val = left.calc()?;
                let right_val = right.calc()?;
                match &op {
                    Operator::Plus => return Some(left_val + right_val),
                    Operator::Minus => return Some(left_val - right_val),
                    Operator::Multiply => return Some(left_val * right_val),
                    Operator::Divide => return Some(left_val / right_val),
                    _ => todo!(),
                };
            }
            Expr::Prefix(op, expr) => {
                let val = expr.calc()?;
                match op {
                    Prefix::Negate => Some(-val),
                    Prefix::Abs => Some(val),
                    Prefix::Not => todo!(), //Some(-val),
                }
            }
            _ => {
                println!("expr calc => {:?}", &self);
                todo!()
            }
        }
    }
}

impl Program {
    pub fn eval(&self) {
        println!("eval LEN:{}", self.statements.len());
        let mut index = 0;
        for expr in &self.statements {
            index += 1;
            match expr {
                _ => {
                    // println!("\x1b[31m eval stmt =>\x1b[39m {:?}",stmt);
                    crate::println(31, "eval expr =>", format!("({:?}) {:?}", index, expr));
                }
            }
        }
    }
}
