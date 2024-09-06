use super::token::TokenPunctuator;

#[derive(Debug)]
pub enum Expr {
    Identifier(String),
    Number(i64),
    
    Infix(Box<Expr>, Infix, Box<Expr>),//算术逻辑 a+b  +-*/
    Call(Box<Expr>, Vec<Expr>),

    Binary(Box<Expr>,TokenPunctuator,Box<Expr>),// a==b
    Expression(Box<Expr>,TokenPunctuator,Expression),

    Prefix(Prefix, Box<Expr>),// !a -1
    Logical(Box<Expr>,Logical,Box<Expr>),// a && b  &&,||,!
}

#[derive(Debug,PartialEq)]
pub enum Expression{
    Assignment,
    Update,
}

#[derive(Debug)]
pub enum Prefix {
    Negate, // -expr
    Not,    // !
}

#[derive(Debug)]
pub enum Infix {
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum Logical{
    ///&&
    And,//&&
    ///||
    Or,//||
    ///!
    Not,
}

#[derive(Debug)]
pub enum Stmt {
    Variable(String, String, Expr),
    Expression(Expr),
    If(),
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
                    Prefix::Not => todo!() //Some(-val),
                }
            }
            _ => {
                println!("expr calc => {:?}",&self);
                todo!()
            },
        }
    }
}


impl Program {
    pub fn eval(&self) {
        println!("LEN:{}",self.statements.len());
        for stmt in &self.statements {
            match stmt {
                Stmt::Variable(kind,name, expr) =>{
                    crate::println(31,"calc =>",format!("{} {} = {}",kind,name,expr.calc().unwrap()));
                },
                _ => {
                    // println!("\x1b[31m eval stmt =>\x1b[39m {:?}",stmt);
                    crate::println(31,"eval stmt =>",format!("{:?}",stmt));
                },
            }
        }
    }
}
