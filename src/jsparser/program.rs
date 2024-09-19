use std::collections::HashMap;

use super::expr::{Expr, Operator};

pub struct Program {
    statements: Vec<Expr>,
    call_map: HashMap<String, Box<dyn Fn(Vec<JSType>)>>, //全局方法
    fn_map: HashMap<String, Expr>,                       //方法
    value_map: HashMap<String, JSType>,                  //全局变量
    call_args: Vec<HashMap<String, JSType>>,             //函数变量,逻辑要优化
    eax: JSType,                                         //存放返回值
}

impl Program {
    pub fn new(statements: Vec<Expr>) -> Self {
        Program {
            statements: statements,
            call_map: HashMap::new(),
            fn_map: HashMap::new(),
            value_map: HashMap::new(),
            call_args: Vec::new(),
            eax: JSType::Void,
        }
    }
    pub fn run(&mut self) {
        self.eval_list(self.statements.clone());
    }
    pub fn register_method(&mut self, ident: String, callback: Box<dyn Fn(Vec<JSType>)>) {
        self.call_map.insert(ident, callback);
    }
    pub fn bind_global_value(&mut self, ident: String, value: JSType) {
        self.value_map.insert(ident, value);
    }
    pub fn bind_call_value(&mut self, ident: String, _args: Option<&Vec<Expr>>) {
        let arg_id = |expr: &Expr| -> String {
            if let Expr::Identifier(a) = expr {
                return a.to_string();
            }
            panic!();
        };

        if None == _args {
            //清空
            self.call_args.remove(self.call_args.len() - 1);
            return;
        }
        let args = _args.unwrap();
        if let Some(call_args) = self.call_args.last_mut() {
            for (index, i) in args.iter().enumerate() {
                let id: String = arg_id(i);
                let value = call_args.get(&format!("{}", index)).unwrap().clone();
                call_args.insert(id, value);
            }
        }
    }
    fn parse(&mut self, mut index: usize, e: &Expr) -> JSType {
        // dbg!(&e);
        match e {
            Expr::Infix(_left, op, _right) => {
                let left = self.parse(index, _left);
                let right = self.parse(index, _right);
                return match &op {
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
                };
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
                if let Some(val) = self.call_args.last() {
                    if let Some(v2) = val.get(t) {
                        return v2.clone();
                    }
                }
                if let Some(val) = self.value_map.get(t) {
                    return val.clone();
                }
                dbg!(&self.call_args);
                panic!("{}", t);
            }
            Expr::Call(ee, expr) => {
                let mut args = HashMap::new();
                let mut arg2 = Vec::new();
                for (i, expr) in expr.iter().enumerate() {
                    if matches!(
                        expr,
                        Expr::Identifier(_) | Expr::Literal(_) | Expr::Call(_, _)
                    ) {
                        let result = self.parse(index, &expr.clone());
                        args.insert(i.to_string(), result.clone());
                        arg2.push(result.clone());
                    } else if matches!(expr, Expr::Infix(_, _, _)) {
                        let result = self.parse(index, &expr);
                        arg2.push(result.clone());
                    } else {
                        dbg!(&expr);
                        panic!()
                    }
                }
                if let Expr::Identifier(ident) = ee.as_ref().clone() {
                    if let Some(e) = self.fn_map.get(&ident) {
                        self.call_args.push(args.clone());
                        let result = self.parse(index, &e.clone());
                        // dbg!(&result);
                        if self.call_args.len() > 0 {
                            self.call_args.remove(self.call_args.len() - 1);
                        }
                        return result;
                    } else if let Some(e) = self.call_map.get(&ident) {
                        // dbg!(&arg2);
                        e(arg2);
                    }
                }
            }
            Expr::Function(ident, args, body) => {
                //处理函数调用后的实现功能
                if let Expr::Identifier(id) = ident.as_ref() {
                    self.bind_call_value(id.clone(), Some(args));
                    let result = self.parse(index, body.as_ref());
                    // dbg!(&result);
                    self.bind_call_value(id.clone(), None); //
                    return result;
                } else {
                    dbg!(&ident);
                    panic!()
                }
            }
            Expr::Return(expr) => match expr.as_ref() {
                Expr::Empty => {
                    return JSType::Void;
                }
                Expr::Infix(_, _, _) => return self.parse(index, &expr),
                _ => {
                    dbg!(&expr);
                    panic!("");
                }
            },
            Expr::BlockStatement(expr) => {
                // dbg!(&expr);
                let mut ret = false;
                for i in expr {
                    if matches!(i, Expr::Return(_)) {
                        ret = true;
                    }
                    let result = self.parse(index, i);
                    if ret {
                        self.eax = result.clone();
                        return result;
                    }
                }
                panic!()
            }
            _ => {
                panic!("  parse => {:?}", e);
            }
        }
        JSType::NULL
    }
    fn eval_list(&mut self, stmt: Vec<Expr>) {
        for (_, expr) in stmt.iter().enumerate() {
            self.eval(0, expr);
        }
    }
    fn eval(&mut self, mut index: usize, stmt: &Expr) {
        match &stmt {
            Expr::Call(ee, vec) => {
                self.parse(index, stmt);
            }
            Expr::If(e, left, right) => {
                let result = self.parse(index, e);
                if let JSType::Bool(r) = result {
                    if r {
                        if let Expr::BlockStatement(expr) = left.as_ref() {
                            self.eval_list(expr.clone());
                        }
                    } else {
                        if let Expr::BlockStatement(expr) = right.as_ref() {
                            self.eval_list(expr.clone());
                        }
                    }
                } else {
                    panic!("{:?}", result);
                }
            }
            Expr::Function(ident, _, _) => {
                //只处理声明函数
                // write(31, format!("({}) {:?}", index + 1, expr));
                if let Expr::Identifier(t) = ident.as_ref() {
                    self.fn_map.insert(t.clone(), stmt.clone());
                } else {
                    panic!("expr::function");
                }
            }
            Expr::BlockStatement(expr) => {
                self.eval_list(expr.clone());
            }
            _ => {
                self.parse(index, stmt);
                // panic!("{:?}", stmt);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum JSType {
    Void, // 无返回值
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
            JSType::Void => Ok("".to_string()),
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
