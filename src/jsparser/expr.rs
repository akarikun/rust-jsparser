#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier(String),
    Number(i64),
    Prefix(Prefix, Box<Expr>),
    Infix(Box<Expr>, Infix, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),

    Binary(Box<Expr>),
    Expression(Box<Expr>),
}

impl Expr {
    pub fn calc(&self) -> Option<i64> {
        match &self {
            Expr::Number(val) => return Some(*val),
            Expr::Infix(left, op, right) => {
                let left_val = left.calc()?;
                let right_val = right.calc()?;
                match &op {
                    Infix::Plus => return Some(left_val + right_val),
                    Infix::Minus => return Some(left_val - right_val),
                    Infix::Multiply => return Some(left_val * right_val),
                    Infix::Divide => return Some(left_val / right_val),
                };
            }
            Expr::Prefix(op, expr) => {
                let val = expr.calc()?;
                match op {
                    Prefix::Negate => Some(-val),
                }
            }
            _ => todo!(),
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
    Literal(String),
    Identifier(String),
    Variable(String, String, Expr),
    Expression(Expr),
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn eval(&self) {
        for stmt in &self.statements {
            // println!("eval stmt => {:?}",stmt);
            match stmt {
                Stmt::Variable(kind,name, expr) =>{
                    let r = expr.calc().unwrap();
                    println!("calc: {} {} = {}",kind,name,r);
                },
                _ => todo!()
            }
        }
    }
}
