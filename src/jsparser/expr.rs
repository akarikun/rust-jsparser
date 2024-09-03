#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier(String),
    Number(i64),
    // Prefix(Prefix, Box<Expr>),
    // Infix(Box<Expr>, Infix, Box<Expr>),
    // Call(Box<Expr>, Vec<Expr>),

    Expression(Box<Expr>),
    Binary(Box<Expr>),
    Variable(Box<Expr>),
}

impl Expr {
    pub fn calc(&self) -> Option<i64> {
        match self {
            // Expr::Number(val) => Some(*val),
            // Expr::Infix(left, op, right) => {
            //     let left_val = left.calc()?;
            //     let right_val = right.calc()?;
            //     match op {
            //         Infix::Plus => Some(left_val + right_val),
            //         Infix::Minus => Some(left_val - right_val),
            //         Infix::Multiply => Some(left_val * right_val),
            //         Infix::Divide => Some(left_val / right_val),
            //     }
            // }
            // Expr::Prefix(op, expr) => {
            //     let val = expr.calc()?;
            //     match op {
            //         Prefix::Negate => Some(-val),
            //     }
            // }
            Expr::Variable(expr)=>{
                Some(0)
            },
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
        // let mut result = None;
        for stmt in &self.statements {
            //result = 
            match stmt {
                Stmt::Variable(kind,name, expr) =>{
                    println!("{} {} {:?}",kind,name, stmt);
                    let t = expr.calc();
                },
                Stmt::Expression(expr) => {
                    expr.calc();
                },
                _ => todo!()
            }
        }
        // println!("{:?}",result);
    }
}
