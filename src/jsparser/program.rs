use std::collections::HashMap;

use super::expr::{Expr, Operator, Variable};

pub struct Program {
    statements: Vec<Expr>,
    global_fn_map: HashMap<String, Box<dyn Fn(Vec<JSType>) -> Result<JSType, String>>>, //外部注册的全局方法
    fn_map: HashMap<String, Expr>,                                                      //方法
    global_value_map: HashMap<String, JSType>, //外部注册的全局变量
    local_value: Vec<HashMap<String, (Variable, JSType)>>, //HashMap<usize, HashMap<String, (Variable, JSType)>>, //变量
    call_value: Vec<Vec<JSType>>,                          //函数变量
}

impl Program {
    pub fn new(statements: Vec<Expr>) -> Self {
        let mut local_value = Vec::new();
        local_value.push(HashMap::new());
        Program {
            statements: statements,
            global_fn_map: HashMap::new(),
            fn_map: HashMap::new(),
            global_value_map: HashMap::new(),
            local_value,
            call_value: Vec::new(),
        }
    }

    pub fn print_tree(&self) {
        println!("/*--------tree--------*/");
        for (index, stmt) in self.statements.iter().enumerate() {
            println!("({}) | {:?}", index + 1, stmt);
        }
        println!("/*-----tree-end------*/");
    }

    pub fn run(&mut self) {
        //需要最先加载所有方法
        for (_, expr) in self.statements.clone().iter().enumerate() {
            match &expr {
                Expr::Function(ident, _, _) => {
                    //首次运行时,需要先注册全局函数
                    if let Expr::Identifier(t) = ident.as_ref() {
                        self.fn_map.insert(t.clone(), expr.clone());
                    } else {
                        panic!("expr::function");
                    }
                }
                _ => {}
            }
        }
        for (_, expr) in self.statements.clone().iter().enumerate() {
            match &expr {
                Expr::Function(_, _, _) => {
                    //这里要跳过function,当call执行时才会调用
                    continue;
                }
                _ => {
                    let result = self.parse(0, expr);
                    match result {
                        Ok(res) => {
                            self.reset_index_value();
                            // println!("{}")
                        }
                        Err(msg) => {
                            println!("\x1b[31m{}\x1b[39m", msg);
                            return;
                        }
                    }
                }
            }
        }
    }
    pub fn register_method(
        &mut self,
        ident: String,
        callback: Box<dyn Fn(Vec<JSType>) -> Result<JSType, String>>,
    ) {
        self.global_fn_map.insert(ident, callback);
    }
    pub fn bind_value(&mut self, ident: String, value: JSType) {
        self.global_value_map.insert(ident, value);
    }

    fn bind_local_arg(
        &mut self,
        index: usize,
        typ: Option<Variable>,
        arg: String,
        value: JSType,
    ) -> Result<(), String> {
        // println!("{:?} {:?}={:?}", typ, arg, value);
        if let Some((v, val)) = self.local_value[index].get_mut(&arg) {
            if typ == None {
                *val = value.clone();
                return Ok(());
            }
            if !(matches!(v.clone(), Variable::Var) && typ.unwrap() == Variable::Var) {
                return Err(self.err("Uncaught TypeError: Assignment to constant variable."));
            } else {
                *val = value.clone();
                return Ok(());
            }
        } else {
            self.local_value[index]
                .entry(arg.clone())
                .or_insert((typ.unwrap_or(Variable::Var), value));
        }
        Ok(())
    }
    fn bind_local_args(&mut self, typ: Variable, args: &Vec<Expr>, values: Vec<JSType>) {
        let mut list: HashMap<String, (Variable, JSType)> = HashMap::new();
        for (index, e) in args.iter().enumerate() {
            match e {
                Expr::Identifier(t) => {
                    list.insert(t.clone(), (typ.clone(), values[index].clone()));
                }
                _ => {
                    panic!("")
                }
            }
        }
        self.local_value.push(list);
    }

    /// index层级变化时调用
    fn update_index(&mut self, index: &mut usize, is_inc: bool) {
        if is_inc {
            *index += 1;
            // self.local_value.insert(*index, HashMap::new());
            self.local_value.push(HashMap::new());
        } else {
            // self.local_value.remove(index);
            // self.local_value.remove(self.local_value.len() - 1);
            *index -= 1;
        }
    }

    fn reset_index_value(&mut self) {
        let len = self.local_value.len();
        for _ in 1..len {
            self.local_value.remove(1);
        }
    }

    pub fn log_value_print(&mut self) {
        dbg!(&self.local_value);
        dbg!(&self.call_value);
    }

    fn err(&self, str: &str) -> String {
        if cfg!(debug_assertions) {
            let msg = format!(
                //"\x1b[31m{}\x1b[39m,token:<\x1b[32m{}\x1b[39m>",
                "{:?}\n{}",
                self.statements.first().unwrap().to_raw(),
                str,
            );
            panic!("\x1b[31m{}\x1b[39m", msg);
        } else {
            let msg = format!(
                //"\x1b[31m{}\x1b[39m,token:<\x1b[32m{}\x1b[39m>",
                "{:?}",
                str,
            );
            return msg;
        }
    }

    fn parse_call_function(
        &mut self,
        mut index: usize,
        values: Vec<JSType>,
        fn_body: Expr,
    ) -> Result<JSType, String> {
        if let Expr::Function(ident, args, body) = fn_body {
            self.update_index(&mut index, true);
            self.bind_local_args(Variable::Var, &args, values); //绑定参数

            // if let Expr::Identifier(id) = ident.as_ref() {
            match body.as_ref().clone() {
                Expr::BlockStatement(vec) => {
                    for i in vec {
                        match i {
                            Expr::Return(expr) => {
                                if let Expr::Function(a, b, c) = expr.as_ref() {
                                    return Ok(JSType::Function(
                                        a.as_ref().clone(),
                                        b.clone(),
                                        c.as_ref().clone(),
                                    ));
                                } else if matches!(expr.as_ref(), Expr::Empty) {
                                    return Ok(JSType::Void);
                                } else if matches!(
                                    expr.as_ref(),
                                    Expr::Identifier(_)
                                        | Expr::Call(_, _)
                                        | Expr::Literal(_, _)
                                        | Expr::Infix(_, _, _)
                                ) {
                                    let result = self.parse(index, &expr)?;
                                    return Ok(result);
                                } else {
                                    dbg!(&expr);
                                    panic!("{:?}", expr);
                                }
                            }
                            _ => {
                                let _expr = self.parse(index, &i);
                                // return _expr;
                            }
                        }
                    }
                }
                _ => panic!("{:?}", body),
            }
            // }
        }
        Ok(JSType::Void)
    }

    ///语法解析及执行，使用递归处理所有语句
    fn parse(&mut self, mut index: usize, e: &Expr) -> Result<JSType, String> {
        match e {
            Expr::Infix(_left, op, _right) => {
                let left = self.parse(index, _left)?;
                let right = self.parse(index, _right)?;
                return match &op {
                    Operator::Plus => left.add(&right),
                    Operator::Subtract => left.subtract(&right),
                    Operator::Multiply => left.multiply(&right),
                    Operator::Divide => left.divide(&right),
                    Operator::Modulo => left.modulo(&right),
                    Operator::Equal => Ok(JSType::Bool(left.equal(&right))),
                    Operator::NE => Ok(JSType::Bool(!left.equal(&right))),
                    Operator::GT => Ok(JSType::Bool(left.GT(&right))),
                    Operator::GTE => Ok(JSType::Bool(left.GTE(&right))),
                    Operator::LT => Ok(JSType::Bool(left.LT(&right))),
                    Operator::LTE => Ok(JSType::Bool(left.LTE(&right))),
                    _ => todo!("{:?}", &op),
                };
            }
            Expr::Literal(val, raw) => {
                if raw.starts_with('"') || raw.starts_with('\'') {
                    return Ok(JSType::String(val.to_string()));
                }
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
                let last_index = index.clone();
                let mut index = (self.local_value.len() - 1) as i32; //index.clone() as i32;
                loop {
                    if let Some(val) = self.local_value.get(index as usize).clone() {
                        if let Some(v) = val.get(t) {
                            return Ok(v.1.clone());
                        }
                    }
                    index -= 1;
                    if index <= -1 {
                        break;
                    }
                }
                if let Some(val) = self.global_value_map.get(t) {
                    return Ok(val.clone());
                }
                if cfg!(debug_assertions) {
                    dbg!(&last_index);
                    dbg!(&self.local_value);
                }
                return Err(self.err(&format!("Uncaught ReferenceError: {} is not defined", t)));
            }
            Expr::Call(ee, args) => {
                match ee.as_ref() {
                    Expr::Identifier(t) => {
                        let mut list = Vec::new();
                        for i in args {
                            list.push(self.parse(index, i)?);
                        }
                        if let Some(e) = self.fn_map.get(&t.clone()) {
                            let expr = self.parse_call_function(index, list, e.clone())?;
                            return Ok(expr);
                        } else if let Some(e) = self.global_fn_map.get(&t.clone()) {
                            let result = e(list)?;
                            return Ok(result);
                        }
                    }
                    Expr::Call(ee2, args2) => {
                        let expr = self.parse(index, ee)?;
                        let mut list = Vec::new();
                        for i in args {
                            list.push(self.parse(index, i)?);
                        }
                        if let JSType::Function(a, b, c) = expr {
                            let body = Expr::Function(Box::new(a), b, Box::new(c));
                            let expr2 = self.parse_call_function(index, list, body)?;
                            // dbg!(&expr2);
                            return Ok(expr2);
                        }
                        return Ok(expr);
                    }
                    _ => {
                        panic!("")
                    }
                };
            }
            Expr::Assignment(ident, value) => {
                let result = self.parse(index, value.as_ref())?;
                self.bind_local_arg(index, Some(Variable::Var), ident.clone(), result);
            }
            Expr::Variable2(v) => {
                for i in v {
                    let result = self.parse(index, &i.2.clone())?;
                    self.bind_local_arg(index, Some(i.0.clone()), i.1.clone(), result);
                }
            }
            Expr::Variable(variable, ident, value) => {
                let result = self.parse(index, value.as_ref())?;
                self.bind_local_arg(index, Some(variable.clone()), ident.clone(), result);
            }
            Expr::For(init, test, update, body) => {
                // println!("{:?}", e);
                self.update_index(&mut index, true);
                let init = self.parse(index, init.as_ref())?;
                loop {
                    let test = self.parse(index, test.as_ref())?;
                    if let JSType::Bool(flag) = test {
                        if flag {
                            self.parse(index, body)?;
                            self.parse(index, update)?;
                        } else {
                            break;
                        }
                    } else {
                        dbg!(&test);
                        return Err(self.err(&format!("表达式异常")));
                    }
                }
                self.update_index(&mut index, false);
            }
            Expr::ForIn(_, _) => {}
            Expr::ForOf(_, _) => {}
            Expr::Update(ident, op, _) => {
                let val = self.parse(index, &ident)?.INC()?;
                if let Expr::Identifier(id) = ident.as_ref() {
                    // dbg!(&index);
                    // dbg!(&val);
                    // dbg!(&id);
                    // dbg!(&val);
                    self.bind_local_arg(index, None, id.clone(), val);
                } else {
                    return Err(self.err(&format!("暂不支持其他表达式")));
                }
            }
            Expr::If(e, left, right) => {
                let result = self.parse(index, e)?;
                if let JSType::Bool(r) = result {
                    if r {
                        self.parse(index, left.as_ref())?;
                    } else {
                        self.parse(index, right.as_ref())?;
                    }
                } else {
                    return Err(self.err(&format!("if解析异常")));
                }
            }
            Expr::Expression(expr) => {
                return self.parse(index, expr);
            }
            Expr::BlockStatement(t) => {
                for i in t {
                    _ = self.parse(index, i);
                }
            }
            // Expr::Return(t)=>{
            //     matches!()
            //     return Ok(JSType::Void)
            // }
            _ => {
                dbg!(&e);
                return Err(self.err(&format!("功能暂未完成,{:?}", e)));
            }
        }
        Ok(JSType::NULL)
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
    Function(Expr, Vec<Expr>, Expr),
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
            JSType::Function(t, _, _) => Ok(format!("function:{}", t.to_raw())),
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
            JSType::Int(t) => Ok(JSType::Int(t + 1)),
            _ => Err(format!(
                "Uncaught SyntaxError: Invalid left-hand side expression in postfix operation"
            )),
        }
    }
    pub fn DEC(&self) -> Result<JSType, String> {
        match self {
            JSType::Int(t) => Ok(JSType::Int(t - 1)),
            _ => Err(format!(
                "Uncaught SyntaxError: Invalid left-hand side expression in postfix operation"
            )),
        }
    }
}
