use std::collections::HashMap;

use super::expr::{Expr, Operator};

pub struct Program {
    pub statements: Vec<Expr>,
    pub call_map: HashMap<String, Box<dyn Fn(Vec<JSType>)>>,
    pub value_map: HashMap<String, JSType>,
}

impl Program {
    pub fn register_method(&mut self, ident: String, callback: Box<dyn Fn(Vec<JSType>)>) {
        self.call_map.insert(ident, callback);
    }
    pub fn bind_value(&mut self, ident: String, value: JSType) {
        self.value_map.insert(ident, value);
    }

    fn parse(&self, e: &Expr) -> JSType {
        match &e {
            Expr::Infix(_left, op, _right) => {
                let left = self.parse(_left);
                let right = self.parse(_right);
                match &op {
                    Operator::Plus => match left.add(&right) {
                        Ok(result) => result,
                        Err(e) => panic!("{}", e),
                    },
                    Operator::Subtract => match left.subtract(&right) {
                        Ok(result) => result,
                        Err(e) => panic!("{}", e),
                    },
                    Operator::Multiply => match left.multiply(&right) {
                        Ok(result) => result,
                        Err(e) => panic!("{}", e),
                    },
                    Operator::Divide => match left.divide(&right) {
                        Ok(result) => result,
                        Err(e) => panic!("{}", e),
                    },
                    Operator::Modulo => match left.modulo(&right) {
                        Ok(result) => result,
                        Err(e) => panic!("{}", e),
                    },
                    Operator::Equal => JSType::Bool(left.equal(&right)),
                    _ => todo!("{:?}", &op),
                }
            }
            Expr::Literal(val) => {
                let i = val.parse::<i64>();
                if !i.is_err() {
                    return JSType::Int(i.unwrap());
                }
                let f = val.parse::<f64>();
                if !f.is_err() {
                    return JSType::Float(f.unwrap());
                }
                return JSType::String(val.clone());
            }
            Expr::Identifier(t) => {
                if let Some(val) = self.value_map.get(t) {
                    return val.clone();
                } else {
                    panic!("{}",t);
                }
            }
            _ => {
                panic!("  parse => {:?}", e);
            }
        }
    }
    pub fn eval(&self, log: bool, statements: Vec<Expr>) {
        let mut stmt: Vec<Expr> = statements;
        if stmt.len() == 0 {
            stmt = self.statements.clone();
        }
        let write = |color: usize, msg: String| {
            if log {
                println!("\x1b[{}m eval expr =>\x1b[39m {}", color, msg);
            }
        };
        write(31, format!("LEN:{}", self.statements.len()));
        for (index, expr) in stmt.iter().enumerate() {
            match &expr {
                Expr::Call(ee, vec) => {
                    write(31, format!("({}) {:?}\n", index + 1, expr));
                    if let Expr::Identifier(ident) = ee.as_ref() {
                        if let Some(e) = self.call_map.get(ident) {
                            let mut v = Vec::new();
                            for i in vec {
                                let result = self.parse(i);
                                v.push(result);
                            }
                            e(v);
                        }
                    }
                }
                Expr::If(e, left, right) => {
                    if let JSType::Bool(r) = self.parse(e) {
                        if r {
                            if let Expr::BlockStatement(v) = left.as_ref() {
                                self.eval(log, v.clone());
                            }
                        } else {
                            if let Expr::BlockStatement(v) = right.as_ref() {
                                self.eval(log, v.clone());
                            }
                        }
                        continue;
                    }
                    panic!()
                }
                _ => {
                    write(31, format!("({}) {:?}", index + 1, expr));
                }
            }
        }
    }
}

#[derive(Debug,Clone)]
pub enum JSType {
    NULL,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl JSType {
    pub fn to_string(&self) -> Result<String, String> {
        match &self {
            JSType::NULL => Err("Cannot read properties of null".to_string()),
            JSType::Int(t) => Ok(t.to_string()),
            JSType::Float(t) => Ok(t.to_string()),
            JSType::String(t) => Ok(t.to_string()),
            JSType::Bool(t) => Ok(t.to_string()),
        }
    }
    pub fn add(&self, other: &JSType) -> Result<JSType, String> {
        match (self, other) {
            (JSType::Int(a), JSType::Int(b)) => Ok(JSType::Int(a + b)),
            (JSType::Float(a), JSType::Float(b)) => Ok(JSType::Float(a + b)),
            (JSType::Int(a), JSType::Float(b)) => Ok(JSType::Float(*a as f64 + b)),
            (JSType::Float(a), JSType::Int(b)) => Ok(JSType::Float(a + *b as f64)),
            (JSType::String(a), JSType::String(b)) => Ok(JSType::String(format!("{},{}", a, b))),
            (JSType::NULL, JSType::String(b)) => Ok(JSType::String(format!("null{}", b))),
            (JSType::Int(a), JSType::String(b)) => Ok(JSType::String(format!("{}{}", a, b))),
            (JSType::Float(a), JSType::String(b)) => Ok(JSType::String(format!("{}{}", a, b))),
            (JSType::String(a), JSType::NULL) => Ok(JSType::String(format!("{}null", a))),
            (JSType::String(a), JSType::Int(b)) => Ok(JSType::String(format!("{}{}", a, b))),
            (JSType::String(a), JSType::Float(b)) => Ok(JSType::String(format!("{}{}", a, b))),
            (JSType::String(a), JSType::Bool(b)) => Ok(JSType::String(format!("{}{}", a, b))),
            (JSType::Bool(a), JSType::String(b)) => Ok(JSType::String(format!("{}{}", a, b))),
            _ => Err("Unsupported types for addition".to_string()),
        }
    }

    pub fn subtract(&self, other: &JSType) -> Result<JSType, String> {
        match (self, other) {
            (JSType::Int(a), JSType::Int(b)) => Ok(JSType::Int(a - b)),
            (JSType::Float(a), JSType::Float(b)) => Ok(JSType::Float(a - b)),
            (JSType::Int(a), JSType::Float(b)) => Ok(JSType::Float(*a as f64 - b)),
            (JSType::Float(a), JSType::Int(b)) => Ok(JSType::Float(a - *b as f64)),
            _ => Err("Unsupported types for subtraction".to_string()),
        }
    }

    pub fn multiply(&self, other: &JSType) -> Result<JSType, String> {
        match (self, other) {
            (JSType::Int(a), JSType::Int(b)) => Ok(JSType::Int(a * b)),
            (JSType::Float(a), JSType::Float(b)) => Ok(JSType::Float(a * b)),
            (JSType::Int(a), JSType::Float(b)) => Ok(JSType::Float(*a as f64 * b)),
            (JSType::Float(a), JSType::Int(b)) => Ok(JSType::Float(a * *b as f64)),
            _ => Err("Unsupported types for multiplication".to_string()),
        }
    }

    pub fn divide(&self, other: &JSType) -> Result<JSType, String> {
        match (self, other) {
            (JSType::Int(a), JSType::Int(b)) => {
                if *b == 0 {
                    Err("Cannot divide by zero".to_string())
                } else {
                    Ok(JSType::Int(a / b))
                }
            }
            (JSType::Float(a), JSType::Float(b)) => {
                if *b == 0.0 {
                    Err("Cannot divide by zero".to_string())
                } else {
                    Ok(JSType::Float(a / b))
                }
            }
            (JSType::Int(a), JSType::Float(b)) => {
                if *b == 0.0 {
                    Err("Cannot divide by zero".to_string())
                } else {
                    Ok(JSType::Float(*a as f64 / b))
                }
            }
            (JSType::Float(a), JSType::Int(b)) => {
                if *b == 0 {
                    Err("Cannot divide by zero".to_string())
                } else {
                    Ok(JSType::Float(a / *b as f64))
                }
            }
            _ => Err("Unsupported types for division".to_string()),
        }
    }

    pub fn modulo(&self, other: &JSType) -> Result<JSType, String> {
        match (self, other) {
            (JSType::Int(a), JSType::Int(b)) => {
                if *b == 0 {
                    Err("Cannot modulo by zero".to_string())
                } else {
                    Ok(JSType::Int(a % b))
                }
            }
            (JSType::Float(a), JSType::Float(b)) => {
                if *b == 0.0 {
                    Err("Cannot modulo by zero".to_string())
                } else {
                    Ok(JSType::Float(a % b))
                }
            }
            (JSType::Int(a), JSType::Float(b)) => {
                if *b == 0.0 {
                    Err("Cannot modulo by zero".to_string())
                } else {
                    Ok(JSType::Float(*a as f64 % b))
                }
            }
            (JSType::Float(a), JSType::Int(b)) => {
                if *b == 0 {
                    Err("Cannot modulo by zero".to_string())
                } else {
                    Ok(JSType::Float(a % *b as f64))
                }
            }
            _ => Err("Unsupported types for modulo".to_string()),
        }
    }

    pub fn equal(&self, other: &JSType) -> bool {
        match (self, other) {
            (JSType::NULL, JSType::NULL) => true,
            (JSType::Int(a), JSType::Int(b)) => a == b,
            (JSType::Int(a), JSType::Float(b)) => *a as f64 == *b,
            (JSType::Int(a), JSType::String(b)) => a.to_string() == b.to_string(),
            (JSType::Int(a), JSType::Bool(b)) => (*a != 0) == *b,
            (JSType::Float(a), JSType::Int(b)) => *a == *b as f64,
            (JSType::Float(a), JSType::Float(b)) => a == b,
            (JSType::Float(a), JSType::String(b)) => a.to_string() == b.to_string(),
            (JSType::Float(a), JSType::Bool(b)) => (*a != 0.0) == *b,
            (JSType::String(a), JSType::Int(b)) => a.to_string() == b.to_string(),
            (JSType::String(a), JSType::Float(b)) => a.to_string() == b.to_string(),
            (JSType::String(a), JSType::String(b)) => a.to_string() == b.to_string(),
            (JSType::String(a), JSType::Bool(b)) => a.to_string() == b.to_string(),
            (JSType::Bool(a), JSType::Int(b)) => (*a && *b == 1) || (!*a && *b == 0),
            (JSType::Bool(a), JSType::Float(b)) => (*a && *b == 1.0) || (!*a && *b == 0.0),
            (JSType::Bool(a), JSType::Bool(b)) => a == b,
            _ => false,
        }
    }
}
