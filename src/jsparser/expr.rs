#[derive(Debug, Clone)]
pub enum Expr {
    Empty, //base
    Identifier(String),
    Number(i64),

    Prefix(Prefix, Box<Expr>),             // !a  -1
    Call(Box<Expr>, Vec<Expr>),            //Box<Expr> => Identifier(String)
    Infix(Box<Expr>, Operator, Box<Expr>), //算术符号 a+b  +-*/   a && b  逻辑符号 &&,||,!
    Update(Box<Expr>, Operator, bool),     //a++/++a     bool:存放++的前后位置
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Assignment,
    Update,
}

#[derive(Debug, Clone)]
pub enum Prefix {
    Negate, // -expr
    Not,    // !
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Variable(String, String, Expr),
    Expression(Expr),
    If(),
    Unexpected(String), //异常
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Expr {
    pub fn calc(&self) -> Option<i64> {
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
        for stmt in &self.statements {
            index += 1;
            match stmt {
                // Stmt::Variable(kind, name, expr) => {
                //     crate::println(
                //         31,
                //         "calc =>",
                //         format!("{} {} = {}", kind, name, expr.calc().unwrap()),
                //     );
                // }
                _ => {
                    // println!("\x1b[31m eval stmt =>\x1b[39m {:?}",stmt);
                    crate::println(31, "eval stmt =>", format!("({:?}) {:?}", index, stmt));
                }
            }
        }
    }
}
