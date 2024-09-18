use std::collections::HashMap;

use super::expr::Expr;

pub struct Program {
    pub statements: Vec<Expr>,
    pub call_map: HashMap<String, Box<dyn Fn(Vec<String>)>>,
}

impl Program {
    pub fn register_method(&mut self, ident: String, callback: Box<dyn Fn(Vec<String>)>) {
        self.call_map.insert(ident, callback);
    }
    pub fn each(&self, e: &Expr) -> String {
        match &e {
            Expr::Infix(_, _, _) => e.calc().unwrap_or_default().to_string(),
            _ => String::from(""),
        }
    }
    pub fn eval(&self, log: bool) {
        let write = |color: usize, msg: String| {
            if log {
                println!("\x1b[{}m eval expr =>\x1b[39m {}", color, msg);
            }
        };
        write(31, format!("LEN:{}", self.statements.len()));
        for (index, expr) in self.statements.iter().enumerate() {
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
                _ => {
                    write(31, format!("({}) {:?}", index + 1, expr));
                }
            }
        }
    }
}
