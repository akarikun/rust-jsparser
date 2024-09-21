use std::collections::HashMap;

use super::expr::{Expr, Operator};

pub struct Program {
    statements: Vec<Expr>,
    global_fn_map: HashMap<String, Box<dyn Fn(Vec<JSType>) -> Result<JSType, String>>>, //外部注册的全局方法
    fn_map: HashMap<String, Expr>,                                                      //方法
    value_map: HashMap<String, JSType>, //外部注册的全局变量
    local_value: HashMap<usize, HashMap<String, JSType>>, //变量
    call_value: Vec<Vec<JSType>>,       //函数变量
}

impl Program {
    pub fn new(statements: Vec<Expr>) -> Self {
        let mut local_value = HashMap::new();
        local_value.insert(0, HashMap::new());
        Program {
            statements: statements,
            global_fn_map: HashMap::new(),
            fn_map: HashMap::new(),
            value_map: HashMap::new(),
            local_value,
            call_value: Vec::new(),
        }
    }

    pub fn print_tree(&self) {
        println!("\n/*--------tree--------*/");
        for (index, stmt) in self.statements.iter().enumerate() {
            println!("({}) | {:?}", index + 1, stmt);
        }
        println!("/*-----tree-end------*/\n");
    }
    ///会初始先加载全局方法
    pub fn run(&mut self) {
        self.init_function(self.statements.clone());
        self.eval_list(self.statements.clone());
    }
    pub fn register_method(
        &mut self,
        ident: String,
        callback: Box<dyn Fn(Vec<JSType>) -> Result<JSType, String>>,
    ) {
        self.global_fn_map.insert(ident, callback);
    }
    pub fn bind_value(&mut self, ident: String, value: JSType) {
        self.value_map.insert(ident, value);
    }
    fn bind_local_args(&mut self, index: usize, args: &Vec<Expr>, is_bind: bool) {
        if !is_bind {
            self.local_value.remove(&index);
        }
        if args.len() == 0 {
            self.local_value.insert(index, HashMap::new());
            return;
        }
        let arr = self.call_value.pop().unwrap();
        let mut map = HashMap::new();
        for (i, e) in args.iter().enumerate() {
            if let Expr::Identifier(t) = e {
                map.insert(t.clone(), arr[i].clone());
            }
        }
        self.local_value.insert(index, map);
    }
    fn bind_local_arg(&mut self, index: usize, ident: String, expr: JSType, is_bind: bool) {
        if !is_bind {
            self.local_value.remove(&index);
        }
        if let Some(val) = self.local_value.get_mut(&index) {
            val.insert(ident, expr.clone());
        } else {
            let mut map = HashMap::new();
            map.insert(ident, expr);
            self.local_value.insert(index, map);
        }
    }
    fn parse(&mut self, mut index: usize, e: &Expr) -> Result<JSType, String> {
        match e {
            Expr::Infix(_left, op, _right) => {
                let left = self.parse(index, _left)?;
                let right = self.parse(index, _right)?;
                return match &op {
                    Operator::Plus => match left.add(&right) {
                        Ok(result) => Ok(result),
                        Err(e) => Err(e),
                    },
                    Operator::Subtract => match left.subtract(&right) {
                        Ok(result) => Ok(result),
                        Err(e) => Err(e),
                    },
                    Operator::Multiply => match left.multiply(&right) {
                        Ok(result) => Ok(result),
                        Err(e) => Err(e),
                    },
                    Operator::Divide => match left.divide(&right) {
                        Ok(result) => Ok(result),
                        Err(e) => Err(e),
                    },
                    Operator::Modulo => match left.modulo(&right) {
                        Ok(result) => Ok(result),
                        Err(e) => Err(e),
                    },
                    Operator::Equal => Ok(JSType::Bool(left.equal(&right))),
                    Operator::NE => Ok(JSType::Bool(!left.equal(&right))),
                    Operator::GT => Ok(JSType::Bool(left.GT(&right))),
                    Operator::GTE => Ok(JSType::Bool(left.GTE(&right))),
                    Operator::LT => Ok(JSType::Bool(left.LT(&right))),
                    Operator::LTE => Ok(JSType::Bool(left.LTE(&right))),
                    _ => todo!("{:?}", &op),
                };
            }
            Expr::Literal(val) => {
                let i = val.parse::<i64>();
                if !i.is_err() {
                    return Ok(JSType::Int(i.unwrap()));
                }
                let f = val.parse::<f64>();
                if !f.is_err() {
                    return Ok(JSType::Float(f.unwrap()));
                }
                return Ok(JSType::String(val.clone()));
            }
            Expr::Identifier(t) => {
                let mut index = index.clone() as i32;
                loop {
                    if let Some(val) = self.local_value.get(&(index as usize)) {
                        if let Some(v) = val.get(t) {
                            return Ok(v.clone());
                        }
                    }
                    index -= 1;
                    if index < -1 {
                        break;
                    }
                }
                if let Some(val) = self.value_map.get(t) {
                    return Ok(val.clone());
                }
                dbg!(&self.local_value);
                panic!("{}", t);
            }
            Expr::Call(ee, expr) => {
                let mut args: Vec<JSType> = Vec::new();
                for (i, expr2) in expr.iter().enumerate() {
                    if matches!(
                        expr2,
                        Expr::Identifier(_) | Expr::Literal(_) | Expr::Call(_, _)
                    ) {
                        let result = self.parse(index, &expr2.clone())?;
                        args.push(result.clone());
                    } else if matches!(expr2, Expr::Infix(_, _, _)) {
                        let result = self.parse(index, &expr2)?;
                        args.push(result);
                    } else {
                        dbg!(&expr2);
                        panic!()
                    }
                }
                if let Expr::Identifier(ident) = ee.as_ref().clone() {
                    self.call_value.push(args.clone());
                    if let Some(e) = self.fn_map.get(&ident) {
                        let result = self.parse(index, &e.clone());
                        // dbg!(&result);
                        if self.local_value.len() > 0 {
                            self.local_value.remove(&(self.local_value.len() - 1));
                        }
                        return result;
                    } else if let Some(e) = self.global_fn_map.get(&ident) {
                        return e(args.clone());
                    } else {
                        return Err(format!("Uncaught ReferenceError: {} is not defined", ident));
                    }
                }
            }
            Expr::Function(ident, args, body) => {
                //处理函数调用后的实现功能
                index += 1;
                self.bind_local_args(index, args, true);
                if let Expr::Identifier(id) = ident.as_ref() {
                    let result = self.parse(index, body.as_ref());
                    // dbg!(&result);
                    return result;
                } else {
                    dbg!(&ident);
                    panic!("功能暂未实现")
                }
                index -= 1;
            }
            Expr::Return(expr) => match expr.as_ref() {
                Expr::Empty => {
                    return Ok(JSType::Void);
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
                        return result;
                    }
                }
            }
            Expr::Assignment(ident, value) => {
                let result = self.parse(index, value.as_ref())?;
                self.bind_local_arg(index, ident.clone(), result, true);
            }
            Expr::Variable(variable, ident, value) => {
                let result = self.parse(index, value.as_ref())?;
                self.bind_local_arg(index, ident.clone(), result, true);
                //还需要判断variable类型
            }
            Expr::For(init, test, update, block) => {
                index += 1;
                let init = self.parse(index, init.as_ref())?;
                loop {
                    let test = self.parse(index, test.as_ref())?;
                    if let JSType::Bool(flag) = test {
                        if flag {
                            self.parse(index, block)?;
                            self.parse(index, update)?;
                        } else {
                            break;
                        }
                    } else {
                        dbg!(&test);
                        return Err(format!("表达式异常"));
                    }
                }
                index -= 1;
            }
            Expr::ForIn(_, _) => {}
            Expr::ForOf(_, _) => {}
            Expr::Update(ident, op, _) => {
                let val = self.parse(index, &ident)?.INC()?;
                if let Expr::Identifier(id) = ident.as_ref() {
                    self.bind_local_arg(index, id.clone(), val, true);
                } else {
                    return Err(format!("暂不支持其他表达式"));
                }
            }
            _ => {
                panic!("  parse => {:?}", e);
            }
        }
        Ok(JSType::NULL)
    }
    fn eval_list(&mut self, stmt: Vec<Expr>) {
        for (_, expr) in stmt.iter().enumerate() {
            if let Some(msg) = self.eval(0, expr) {
                println!("\x1b[31m{}\x1b[39m", msg);
                return;
            }
        }
    }
    fn eval(&mut self, mut index: usize, stmt: &Expr) -> Option<String> {
        match &stmt {
            Expr::Call(_, _) => {
                let result = self.parse(index, stmt);
                if let Err(msg) = result {
                    return Some(msg);
                } else {
                    // dbg!(&result);
                    return None;
                }
            }
            Expr::If(e, left, right) => {
                if let Ok(result) = self.parse(index, e) {
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
                } else {
                }
            }
            Expr::Function(_, _, _) => {
                //这里要跳过方法声明
                return None;
            }
            Expr::BlockStatement(expr) => {
                self.eval_list(expr.clone());
            }
            _ => {
                self.parse(index, stmt);
            }
        }
        None
    }

    ///需要最先加载方法
    fn init_function(&mut self, stmt: Vec<Expr>) {
        for (_, expr) in stmt.iter().enumerate() {
            match &expr {
                Expr::Function(ident, _, _) => {
                    //只处理声明函数
                    if let Expr::Identifier(t) = ident.as_ref() {
                        self.fn_map.insert(t.clone(), expr.clone());
                    } else {
                        panic!("expr::function");
                    }
                }
                _ => {}
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
    /// >
    pub fn GT(&self, other: &JSType) -> bool {
        match (self, other) {
            (JSType::NULL, JSType::NULL) => false,
            (JSType::Int(a), JSType::Int(b)) => a > b,
            (JSType::Int(a), JSType::Float(b)) => *a as f64 > *b,
            // (JSType::Int(a), JSType::String(b)) => false,
            (JSType::Int(a), JSType::Bool(b)) => (*a != 0) == *b,
            (JSType::Float(a), JSType::Int(b)) => *a == *b as f64,
            (JSType::Float(a), JSType::Float(b)) => a == b,
            // (JSType::Float(a), JSType::String(b)) => false,
            (JSType::Float(a), JSType::Bool(b)) => (*a != 0.0) == *b,
            // (JSType::String(a), JSType::Int(b)) => false,
            // (JSType::String(a), JSType::Float(b)) => false,
            (JSType::String(a), JSType::String(b)) => a == b,
            // (JSType::String(a), JSType::Bool(b)) => false,
            // (JSType::Bool(a), JSType::Int(b)) => false,
            // (JSType::Bool(a), JSType::Float(b)) => false,
            (JSType::Bool(a), JSType::Bool(b)) => a == b,
            // (JSType::Bool(a), JSType::String(b)) => false,
            _ => panic!("暂不支持其他操作符"),
        }
    }
    /// >=
    pub fn GTE(&self, other: &JSType) -> bool {
        self.GT(other) || self.equal(other)
    }
    /// <
    pub fn LT(&self, other: &JSType) -> bool {
        !self.GTE(other)
    }
    /// <=
    pub fn LTE(&self, other: &JSType) -> bool {
        !self.GT(other)
    }

    pub fn INC(&self) -> Result<JSType, String> {
        match self {
            // JSType::Void =>Err(format!("Uncaught SyntaxError: Invalid left-hand side expression in postfix operation")),
            // JSType::NULL =>Err(format!("Uncaught SyntaxError: Invalid left-hand side expression in postfix operation")),
            JSType::Int(t) => Ok(JSType::Int(t + 1)),
            JSType::Float(t) => Ok(JSType::Float(t + 1.0)),
            // JSType::String(_) => todo!(),
            // JSType::Bool(_) => todo!(),//
            _ => Err(format!(
                "Uncaught SyntaxError: Invalid left-hand side expression in postfix operation"
            )),
        }
    }
    pub fn DEC(&self) -> Result<JSType, String> {
        match self {
            // JSType::Void =>Err(format!("Uncaught SyntaxError: Invalid left-hand side expression in postfix operation")),
            // JSType::NULL =>Err(format!("Uncaught SyntaxError: Invalid left-hand side expression in postfix operation")),
            JSType::Int(t) => Ok(JSType::Int(t - 1)),
            JSType::Float(t) => Ok(JSType::Float(t - 1.0)),
            // JSType::String(_) => todo!(),
            // JSType::Bool(_) => todo!(),//
            _ => Err(format!(
                "Uncaught SyntaxError: Invalid left-hand side expression in postfix operation"
            )),
        }
    }
}
