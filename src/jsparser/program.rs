use std::collections::HashMap;

use super::expr::{Expr, Operator};

pub struct Program {
    pub statements: Vec<Expr>,
    pub call_map: HashMap<String, Box<dyn Fn(Vec<JSType>)>>,
}

impl Program {
    pub fn register_method(&mut self, ident: String, callback: Box<dyn Fn(Vec<JSType>)>) {
        self.call_map.insert(ident, callback);
    }
    pub fn calc(&self, expr: &Expr) -> Option<f64> {
        match &expr {
            Expr::Number(val) => return Some(*val),
            Expr::Infix(left, op, right) => {
                let left_val = self.calc(left)?;
                let right_val = self.calc(right)?;
                match &op {
                    Operator::Plus => return Some(left_val + right_val),
                    Operator::Minus => return Some(left_val - right_val),
                    Operator::Multiply => return Some(left_val * right_val),
                    Operator::Divide => return Some(left_val / right_val),
                    Operator::Equal => {
                        if left_val == right_val {
                            return Some(1.0);
                        } else {
                            return Some(0.0);
                        }
                    }
                    _ => todo!("{:?}", &op),
                };
            }
            _ => {
                println!("expr calc => {:?}", &expr);
                todo!()
            }
        }
    }
    fn each(&self, e: &Expr) -> JSType {
        match &e {
            Expr::Infix(left, op, right) => {
                let left_val = self.calc(left).unwrap();
                let right_val = self.calc(right).unwrap();
                match &op {
                    Operator::Plus => return JSType::Float(left_val + right_val),
                    Operator::Minus => return JSType::Float(left_val - right_val),
                    Operator::Multiply => return JSType::Float(left_val * right_val),
                    Operator::Divide => return JSType::Float(left_val / right_val),
                    Operator::Equal => return JSType::Bool(left_val == right_val),
                    _ => todo!("{:?}", &op),
                };
            }
            Expr::Number(t) => JSType::Float(t.clone()),
            Expr::Identifier(t) => JSType::String(t.clone()),
            _ => {
                // println!("  each => {:?}", e);
                JSType::Default
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
                                let result = self.each(i);
                                v.push(result);
                            }
                            e(v);
                        }
                    }
                }
                Expr::If(e, left, right) => {
                    if let JSType::Bool(r) = self.each(e) {
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

#[derive(Debug)]
pub enum JSType {
    Default,
    Integer(i32),
    Float(f64),
    String(String),
    Bool(bool),
}
