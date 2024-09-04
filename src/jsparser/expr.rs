use super::token::TokenPunctuator;

#[derive(Debug)]
pub enum Expr {
    Identifier(String),
    Number(i64),
    Prefix(Prefix, Box<Expr>),
    Infix(Box<Expr>, Infix, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),

    Binary(Box<Expr>),
    Expression(Box<Expr>,TokenPunctuator,Expression),
}

#[derive(Debug,PartialEq)]
pub enum Expression{
    Assignment,
    Update,
}

#[derive(Debug)]
pub enum Prefix {
    Negate, // -expr
}

#[derive(Debug)]
pub enum Infix {
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum Stmt {
    Literal(String),
    Identifier(String),
    Variable(String, String, Expr),
    Expression(Expr),
    // Update(String,Expr),
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
        for stmt in &self.statements {
            match stmt {
                Stmt::Variable(kind,name, expr) =>{
                    println!("calc: {} {} = {}",kind,name,expr.calc().unwrap());
                },
                // Stmt::Expression(expr)=>{
                //     if let Expr::Expression(ident,tp,ex) = expr {
                //         if Expression::Update == *ex {
                //             if TokenPunctuator::INC== *tp {
                //                 //ident.as_ref()
                //             }
                //         }
                //     }
                // },
                _ => {
                    println!("eval stmt => {:?}",stmt);
                },
            }
        }
    }
}
