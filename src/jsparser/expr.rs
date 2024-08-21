#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier(String),
    Number(i64),
    Prefix(Prefix, Box<Expr>),
    Infix(Box<Expr>, Infix, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
}

impl Expr {
    pub fn eval(&self) -> Option<i64> {
        match self {
            Expr::Number(val) => Some(*val),
            Expr::Infix(left, op, right) => {
                let left_val = left.eval()?;
                let right_val = right.eval()?;
                match op {
                    Infix::Plus => Some(left_val + right_val),
                    Infix::Minus => Some(left_val - right_val),
                    Infix::Multiply => Some(left_val * right_val),
                    Infix::Divide => Some(left_val / right_val),
                }
            }
            Expr::Prefix(op, expr) => {
                let val = expr.eval()?;
                match op {
                    Prefix::Negate => Some(-val),
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Prefix {
    Negate, // -expr
}

#[derive(Debug, PartialEq)]
pub enum Infix {
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Let(String, Expr),
    Expression(Expr),
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn eval(&self) -> Option<i64> {
        let mut result = None;
        for stmt in &self.statements {
            result = match stmt {
                Stmt::Let(_, expr) => expr.eval(),
                Stmt::Expression(expr) => expr.eval(),
            }
        }
        result
    }
}
