use std::{cell::RefCell, f32::consts::E, io::SeekFrom, rc::Rc, vec};

use super::{
    expr::{Expr, Operator, Unary, Variable},
    lexer::{ILexer, TokenList},
    program::Program,
    token::{Token, TokenKeyword, TokenPunctuator, TokenType},
};

pub struct Parser {
    lexer: Box<dyn ILexer>,
    current_token: Token,
    peek_token: Token,

    paren: usize,
    cparen: usize,
    sparen: usize,
}

impl Parser {
    pub fn new(lexer: Box<dyn ILexer>) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::new(TokenType::EOF, 0, 0),
            peek_token: Token::new(TokenType::EOF, 0, 0),
            paren: 0,
            cparen: 0,
            sparen: 0,
        };
        parser.next_token();
        parser.next_token();
        parser
    }
    pub fn parse_program(&mut self) -> Result<Program, String> {
        let statements = self.filter_statement(0, true)?;
        Ok(Program::new(statements))
    }

    fn checked_base(&mut self, typ: &TokenType) -> Result<Expr, String> {
        match &self.current_token.typ {
            TokenType::Ident(t) => {
                if self.peek_token.is_ptor(TokenPunctuator::LParen)
                    || self.peek_token.is_ptor(TokenPunctuator::Dot)
                    || self.peek_token.is_ptor(TokenPunctuator::LSParen)
                {
                    let expr = self.parse_call_or_member()?;
                    // dbg!(&expr);
                    Ok(expr)
                } else {
                    let expr = Expr::Identifier(t.clone());
                    self.next_token();
                    Ok(expr)
                }
            }
            TokenType::Literal(t) => {
                let expr = Expr::Literal(t.trim_matches('"').to_string(), t.clone());
                self.next_token();
                Ok(expr)
            }
            TokenType::Punctuator(t) => {
                if t.is_precedence() {
                    // let token = self.next_token();
                    // return self.checked_base(&token.typ);
                    panic!("")
                }
                if matches!(t, TokenPunctuator::LParen) {
                    self.next_token(); //(
                    self.paren += 1;
                    let expr = self.parse(true)?;
                    self.next_token(); //)
                    return Ok(expr);
                }
                if matches!(t, TokenPunctuator::LSParen) {
                    self.next_token(); //[
                    self.sparen += 1;
                    let expr = self.parse(true)?;
                    self.next_token(); //]
                    return Ok(expr);
                }
                if matches!(t, TokenPunctuator::Dot) {
                    // self.next_token(); //[
                    // self.sparen += 1;
                    // let expr = self.parse(true)?;
                    // self.next_token(); //]
                    // return Ok(expr);
                    panic!("")
                }
                return Err(self.err("未知解析"));
            }
            _ => return Err(self.err("未知解析")),
        }
    }

    fn parser_infix(&mut self, precedence: Precedence) -> Result<Expr, String> {
        let mut left = self.checked_base(&self.current_token.typ.clone())?;
        while precedence < self.get_precedence(self.current_token.typ.clone()) {
            let _op = self.next_token();
            let op = self.get_operator(&_op);

            let right = self.checked_base(&self.current_token.typ.clone())?;
            match &left {
                Expr::Identifier(_) | Expr::Literal(_, _) => {
                    left = Expr::Infix(Box::new(left), op, Box::new(right));
                }
                Expr::Infix(left2, op2, right2) => {
                    if self.get_precedence_by_operator(&op2.clone())
                        < self.get_precedence(_op.typ.clone())
                    {
                        left = Expr::Infix(
                            left2.clone(),
                            op2.clone(),
                            Box::new(Expr::Infix(right2.clone(), op, Box::new(right))),
                        );
                    } else {
                        left = Expr::Infix(Box::new(left), op, Box::new(right));
                    }
                }
                _ => panic!(),
            }
        }
        Ok(left)
    }

    fn checked_paren(&mut self, typ: &TokenType) -> Result<bool, String> {
        match typ {
            TokenType::Punctuator(ptor) => {
                if matches!(ptor, TokenPunctuator::RParen) {
                    self.paren -= 1;
                    if self.paren < 0 {
                        return Err(self.err("Unexpected token"));
                    }
                    self.next_token();
                    return Ok(true);
                } else if matches!(ptor, TokenPunctuator::RCParen) {
                    self.cparen -= 1;
                    if self.cparen < 0 {
                        return Err(self.err("Unexpected token"));
                    }
                    self.next_token();
                    return Ok(true);
                } else if matches!(ptor, TokenPunctuator::RSParen) {
                    dbg!(&self.sparen);
                    self.sparen -= 1;
                    if self.sparen < 0 {
                        return Err(self.err("Unexpected token"));
                    }
                    self.next_token();
                    return Ok(true);
                }
            }
            _ => return Ok(false),
        }

        return Ok(false);
    }

    fn parse_base_typ(&mut self, token: &Token) -> Result<Expr, String> {
        if token.is_unary() {
            let unary = self.parse_unary().unwrap();
            self.next_token();
            let expr = self.parse(true).unwrap();
            return Ok(Expr::Unary(unary, Box::new(expr)));
        }
        if token.is_ident() {
            return Ok(Expr::Identifier(token.raw.clone()));
        } else if token.is_literal() {
            if token.raw.starts_with('"') | token.raw.starts_with('\'') {
                return Ok(Expr::Literal2(token.raw.clone()));
            } else {
                return Ok(Expr::Literal(token.raw.clone(), token.raw.clone()));
            }
        }
        /*else if token.is_template_literal() {
            return Expr::TemplateLiteral(token.raw.clone());
        }*/
        else {
            Ok(Expr::Empty)
        }
    }
    /// 解析入口
    fn parse(&mut self, is_skip_semicolon: bool) -> Result<Expr, String> {
        // let is_skip_semicolon = true;
        let skip_semicolon = |p: &mut Self| {
            if p.current_token.is_ptor(TokenPunctuator::Semicolon) {
                if is_skip_semicolon {
                    p.next_token();
                }
                return true;
            }
            false
        };
        _ = match &self.current_token.typ {
            TokenType::Illegal => Err(self.err("Illegal")),
            TokenType::EOF => Ok(Expr::Empty),
            TokenType::Literal(t) => {
                let expr = self.parser_infix(Precedence::Lowest)?;
                skip_semicolon(self);
                return Ok(expr);
            }
            TokenType::Ident(t) => {
                let ident = t.clone();
                if self.peek_token.is_eof(true) {
                    self.next_token();
                    self.next_token();
                    return Ok(Expr::Identifier(ident.clone()));
                }
                if self.peek_token.checked_keyword() {
                    if self.current_token.line == self.peek_token.line {
                        return Err(self.err("Unexpected token"));
                    }
                    self.next_token();
                    return Ok(Expr::Identifier(ident.clone()));
                }
                if self.peek_token.is_ptor(TokenPunctuator::Comma) {
                    self.next_token();
                    return Ok(Expr::Identifier(ident.clone()));
                }
                if self.peek_token.is_ptor(TokenPunctuator::MOV) {
                    //a=<base_Expr>
                    let ident = self.next_token();
                    self.next_token();
                    if self.current_token.is_eof(true) {
                        return Err(self.err("Unexpected token"));
                    }
                    let mut v = Vec::new();
                    let expr = self.parse(is_skip_semicolon)?;
                    v.push((ident.raw.clone(), expr));
                    loop {
                        if self.current_token.is_ptor(TokenPunctuator::Comma) {
                            self.next_token();
                        } else {
                            break;
                        }
                        let expr = self.parse(is_skip_semicolon)?;
                        if let Expr::Assignment2(t) = expr {
                            for i in t {
                                v.push(i);
                            }
                        } else if let Expr::Identifier(t) = expr {
                            v.push((t, Expr::Empty));
                        }
                    }
                    return Ok(Expr::Assignment2(v));
                } else if self.peek_token.is_complex() {
                    //<base_Expr> 符合: 'a+' , 'a[' , 'a(' , 'a.'
                    let expr = self.parser_infix(Precedence::Lowest)?;
                    skip_semicolon(self);
                    return Ok(expr);
                } else if self.peek_token.is_ptor(TokenPunctuator::INC) {
                    //a++
                    self.next_token();
                    self.next_token();
                    return Ok(Expr::Update(
                        Box::new(Expr::Identifier(ident.clone())),
                        format!("++"),
                        false,
                    ));
                } else if self.peek_token.is_ptor(TokenPunctuator::DEC) {
                    //a--
                    self.next_token();
                    self.next_token();
                    return Ok(Expr::Update(
                        Box::new(Expr::Identifier(ident.clone())),
                        format!("--"),
                        false,
                    ));
                }

                let cur = self.current_token.clone();
                let chk = self.checked_paren(&self.peek_token.typ.clone())?;
                if chk {
                    let expr = self.parse_base_typ(&cur)?;
                    return Ok(expr);
                }
                panic!()
            }
            TokenType::Punctuator(t) => {
                let cur = self.current_token.clone();
                if matches!(t, TokenPunctuator::Semicolon) {
                    let expr = self.parse_base_typ(&cur)?;
                    return Ok(expr);
                }
                if matches!(t, TokenPunctuator::Comma) {
                    let expr = self.parse_base_typ(&cur)?;
                    return Ok(expr);
                }
                if cur.is_unary() {
                    let unary = self.parse_unary()?;
                    self.next_token();
                    let expr = self.parse(is_skip_semicolon)?;
                    return Ok(Expr::Unary(unary, Box::new(expr)));
                }
                let chk = self.checked_paren(&self.current_token.typ.clone())?;
                if chk {
                    let expr = self.parse_base_typ(&cur)?;
                    return Ok(expr);
                }
                panic!()
            }
            TokenType::Keyword(t) => {
                //let a = <Expr>;
                if matches!(
                    t,
                    TokenKeyword::Let | TokenKeyword::Var | TokenKeyword::Const
                ) {
                    let token = self.next_token();
                    if !self.current_token.is_ident() {
                        return Err(format!("{}", self.err("Unexpected end of input")));
                    }
                    let key = if token.raw == "var" {
                        Variable::Var
                    } else if token.raw == "let" {
                        Variable::Let
                    } else {
                        Variable::Const
                    };

                    let mut v = Vec::new();
                    loop {
                        let expr = self.parse(is_skip_semicolon)?;
                        if let Expr::Assignment2(ass) = expr {
                            for i in ass {
                                v.push((key.clone(), i.0, i.1));
                            }
                        }
                        if self.current_token.is_ptor(TokenPunctuator::Comma) {
                            self.next_token(); // ,
                        } else {
                            break;
                        }
                    }
                    skip_semicolon(self);
                    return Ok(Expr::Variable2(v));
                } else if matches!(t, TokenKeyword::If) {
                    return self.parse_if_slot();
                } else if matches!(t, TokenKeyword::Else) {
                    return Err(self.err("Unexpected token else"));
                } else if matches!(t, TokenKeyword::For) {
                    return self.parse_for_slot();
                } else if matches!(t, TokenKeyword::Break) {
                    self.next_token();
                    return Ok(Expr::Break);
                } else if matches!(t, TokenKeyword::Continue) {
                    self.next_token();
                    return Ok(Expr::Continue);
                } else if matches!(t, TokenKeyword::Return) {
                    self.next_token();
                    if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                        self.next_token();
                        return Ok(Expr::Return(Box::new(Expr::Empty)));
                    } else {
                        let expr = self.parse(is_skip_semicolon)?;
                        return Ok(Expr::Return(Box::new(expr)));
                    }
                } else if matches!(t, TokenKeyword::Function) {
                    return self.parse_function_slot();
                }
                todo!("{:?}", t);
            }
        };
        Err(self.err("未知解析"))
    }

    //这里还要过滤一次
    fn filter_statement(
        &mut self,
        count: usize,
        is_skip_semicolon: bool,
    ) -> Result<Vec<Expr>, String> {
        let mut statements: Vec<Expr> = Vec::new();
        while self.current_token.typ != TokenType::EOF {
            let result = self.parse(true);
            match result {
                Ok(expr) => match &expr {
                    Expr::Return(_) | Expr::Break | Expr::Continue => {
                        return Err(format!("{}", self.err("Unexpected token")));
                    }
                    _ => {
                        statements.push(expr.clone());
                    }
                },
                Err(msg) => return Err(msg),
            }
        }
        Ok(statements)
    }
    fn log(&self) {
        self.println(
            31,
            format!(
                "<{}>,\n     <{}>",
                &self.current_token.desc(),
                &self.peek_token.desc()
            ),
        );
    }

    fn println(&self, color: i32, msg: String) {
        println!("\x1b[{}m{} \x1b[39m ", color, msg);
    }

    fn err(&self, str: &str) -> String {
        let msg = format!(
            //"\x1b[31m{}\x1b[39m,token:<\x1b[32m{}\x1b[39m>",
            "({}){}",
            self.current_token.desc(),
            str,
        );
        panic!("{}",msg);
        return msg;
    }

    fn next_token(&mut self) -> Token {
        let token = self.current_token.clone();
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
        token
    }

    fn parse_body(&mut self, allow_abbr: bool) -> Result<Expr, String> {
        if !allow_abbr && !self.current_token.is_ptor(TokenPunctuator::LCParen) {
            return Err(self.err("Unexpected token"));
        }
        if self.current_token.is_ptor(TokenPunctuator::LCParen) {
            self.next_token(); // {

            let mut v = Vec::new();
            loop {
                if self.current_token.is_eof(true){
                    break;
                }
                if self.current_token.is_ptor(TokenPunctuator::RCParen) {
                    break;
                }
                let expr = self.parse(true)?;
                v.push(expr);
                // if matches!(expr, Expr::Assignment2(_) | Expr::Call(_, _)) {
                //     v.push(expr);
                // } else {
                //     dbg!(&expr);
                //     return Err(self.err("Use of future reserved word in strict mode"));
                // }
            }
            if !self.current_token.is_ptor(TokenPunctuator::RCParen) {
                return Err(self.err("Unexpected token"));
            }
            self.next_token();//}
            return Ok(Expr::BlockStatement(v));
        } else {
            let expr = self.parse(false)?;
            if matches!(expr, Expr::Assignment2(_) | Expr::Call(_, _)) {
                return Ok(expr);
            } else {
                dbg!(&expr);
                return Err(self.err("Use of future reserved word in strict mode"));
            }
        }
    }

    fn parse_function_slot(&mut self) -> Result<Expr, String> {
        self.next_token(); // function
        if !self.current_token.is_ident() {
            return Err(format!("{}", self.err("Unexpected token")));
        }
        let ident = self.next_token(); //ident
        if !self.current_token.is_ptor(TokenPunctuator::LParen) {
            return Err(format!("{}", self.err("Unexpected token")));
        }
        self.next_token(); //(
        self.paren += 1;
        let mut args = Vec::new();
        loop {
            if self.current_token.is_ptor(TokenPunctuator::RParen) {
                break;
            }
            let expr = self.parse(false)?;
            args.push(expr);
            if self.current_token.is_ptor(TokenPunctuator::Comma) {
                self.next_token();
            } else {
                break;
            }
        }
        if !self.current_token.is_ptor(TokenPunctuator::RParen) {
            return Err(self.err("Unexpected end of input"));
        }
        self.next_token(); //)
        // dbg!(&args);
        let body = self.parse_body(false)?;
        Ok(Expr::Function(
            Box::new(Expr::Identifier(ident.raw.clone())),
            args,
            Box::new(body),
        ))
    }

    fn parse_for_slot(&mut self) -> Result<Expr, String> {
        self.next_token(); //for
        if !self.current_token.is_ptor(TokenPunctuator::LParen) {
            return Err(self.err("Unexpected token"));
        }
        self.next_token(); //(

        // Expr::For((), (), (), ())
        let mut v = Vec::new();
        for _ in 0..3 {
            if v.len() == 2 && self.current_token.is_ptor(TokenPunctuator::RParen) {
                //for(;;)
                v.push(Expr::Empty);
                break;
            }
            let binary = self.parse(false)?;
            if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
            }
            v.push(binary);
        }
        if !self.current_token.is_ptor(TokenPunctuator::RParen) {
            return Err(self.err("Unexpected token"));
        }
        self.next_token(); //)

        let line = self.current_token.line;
        let body = self.parse_body(true)?;

        if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
            self.next_token();
        } else if line == self.current_token.line {
            return Err(self.err("Unexpected token"));
        }
        Ok(Expr::For(
            Box::new(v[0].clone()),
            Box::new(v[1].clone()),
            Box::new(v[2].clone()),
            Box::new(body),
        ))
    }
    fn parse_if_slot(&mut self) -> Result<Expr, String> {
        self.next_token(); //skip if
        if !self.current_token.is_ptor(TokenPunctuator::LParen) {
            return Err(self.err("Unexpected token"));
        }
        self.next_token(); //(
        let binary = self.parse(false)?;

        if !self.current_token.is_ptor(TokenPunctuator::RParen) {
            return Err(self.err("Unexpected token"));
        }
        self.next_token(); //)

        let line = self.current_token.line;
        let left_expr = self.parse_body(true)?;

        if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
            self.next_token();
        } else if line == self.current_token.line {
            return Err(self.err("Unexpected token"));
        }

        if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
            // line = 0;
            self.next_token();
        } else {
            // line = self.current_token.line;
        }

        let mut right_expr = Expr::Empty;
        if self.current_token.is_keyword(TokenKeyword::Else) {
            if line == self.current_token.line {
                return Err(self.err("Unexpected token else"));
            }
            self.next_token(); //else
            if self.current_token.is_keyword(TokenKeyword::If) {
                let expr = self.parse_if_slot()?;
                return Ok(Expr::If(
                    Box::new(binary),
                    Box::new(left_expr),
                    Box::new(expr),
                ));
            } else {
                let line = self.current_token.line;
                right_expr = self.parse_body(true)?;

                if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                    self.next_token();
                } else if line == self.current_token.line {
                    return Err(self.err("Unexpected token"));
                }
            }
        }
        Ok(Expr::If(
            Box::new(binary),
            Box::new(left_expr),
            Box::new(right_expr),
        ))
    }

    fn parse_call_slot(&mut self, callee: &mut Expr) -> Result<Expr, String> {
        self.next_token(); //(
        self.paren += 1;
        if self.current_token.is_ptor(TokenPunctuator::RParen) {
            self.next_token(); //)
            self.paren -= 1;
            return Ok(Expr::Call(Box::new(callee.clone()), Vec::new()));
        }

        let mut v = Vec::new();
        let expr = self.parse(true)?;
        v.push(expr);
        while self.current_token.is_ptor(TokenPunctuator::Comma) {
            self.next_token();
            let expr = self.parse(true)?;
            v.push(expr);
        }
        if !self.current_token.is_ptor(TokenPunctuator::RParen) {
            return Err(self.err("Unexpected end of input"));
        }
        self.next_token(); //)
        return Ok(Expr::Call(Box::new(callee.clone()), v));
    }
    fn parse_member_slot(&mut self, mem: &mut Expr) -> Result<(), String> {
        let t = self.next_token(); // . or [
        if t.is_ptor(TokenPunctuator::Dot) {
            if self.current_token.is_ident() {
                let ident = self.next_token();
                if let Expr::Member(_, property) = mem {
                    *property = Box::new(Expr::Identifier(ident.raw));
                    return Ok(());
                }
            }
        } else if t.is_ptor(TokenPunctuator::LSParen) {
            self.sparen += 1;
            let expr = self.parse(true)?;
            let mut v = Vec::new();
            v.push(expr);
            while self.current_token.is_ptor(TokenPunctuator::Comma) {
                self.next_token();
                let e = self.parse(true)?;
                v.push(e);
            }
            if v.len() == 1 {
                if let Expr::Member(_, property) = mem {
                    *property = Box::new(v.get(0).unwrap().clone());
                } else {
                    return Err(format!("{}", self.err("未解析或异常")));
                }
            } else {
                if let Expr::Member(_, property) = mem {
                    *property = Box::new(Expr::Sequence(v));
                } else {
                    return Err(format!("{}", self.err("未解析或异常")));
                }
            }
            if !self.current_token.is_ptor(TokenPunctuator::RSParen) {
                return Err(format!("{}", self.err("Unexpected end of input")));
            }
            self.next_token(); //]
        }
        Ok(())
    }
    /// 这里还要处理多级 如: a()[1]  a[1]()    a[1]()[1]()...
    fn parse_call_or_member(&mut self) -> Result<Expr, String> {
        let token = self.next_token(); //ident
        let mut expr = Expr::Empty;
        loop {
            if self.current_token.is_eof(true) {
                break;
            }
            if self.current_token.is_ptor(TokenPunctuator::LParen) {
                //(
                if Expr::Empty == expr {
                    expr = Expr::Identifier(token.raw.clone());
                } else {
                    expr = self.parse_call_slot(&mut expr)?;
                    if self.paren == 0 {
                        //(的值为0时表示方法截取完闭，如果后面还有()或[]则还要继续在循环中执行
                        continue;
                    }
                    break;
                }
            } else if self.current_token.is_ptor(TokenPunctuator::LSParen)
                || self.current_token.is_ptor(TokenPunctuator::Dot)
            {
                //[
                if Expr::Empty == expr {
                    expr = Expr::Member(
                        Box::new(Expr::Identifier(token.raw.clone())),
                        Box::new(Expr::Empty),
                    );
                } else {
                    expr = Expr::Member(Box::new(expr), Box::new(Expr::Empty));
                }
                let _ = self.parse_member_slot(&mut expr);
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Unary, String> {
        match &self.current_token.typ {
            TokenType::Punctuator(t) => match &t {
                TokenPunctuator::Not => Ok(Unary::Not),
                TokenPunctuator::Plus => Ok(Unary::Plus),
                TokenPunctuator::Minus => Ok(Unary::Minus),
                TokenPunctuator::BitNot => Ok(Unary::BitNot),
                _ => Err(self.err("Unexpected token")),
            },
            _ => Err(self.err("Unexpected token")),
        }
    }
}

trait IParse {
    fn get_operator(&self, token: &Token) -> Operator;
    fn get_precedence(&self, typ: TokenType) -> Precedence;
    fn get_precedence_by_operator(&self, typ: &Operator) -> Precedence;
}

impl IParse for Parser {
    fn get_operator(&self, token: &Token) -> Operator {
        let op = match &token.typ {
            TokenType::Punctuator(t) => match t {
                TokenPunctuator::Plus => Operator::Plus,
                TokenPunctuator::Minus => Operator::Subtract,
                TokenPunctuator::Multiply => Operator::Multiply,
                TokenPunctuator::Divide => Operator::Divide,
                TokenPunctuator::Modulo => Operator::Modulo,
                TokenPunctuator::Or => Operator::Or,
                TokenPunctuator::And => Operator::And,
                TokenPunctuator::Not => Operator::Not,
                TokenPunctuator::LShift => Operator::LShift,
                TokenPunctuator::RShift => Operator::RShift,
                TokenPunctuator::Equal => Operator::Equal,
                TokenPunctuator::NE => Operator::NE,
                TokenPunctuator::GT => Operator::GT,
                TokenPunctuator::GTE => Operator::GTE,
                TokenPunctuator::LT => Operator::LT,
                TokenPunctuator::LTE => Operator::LTE,
                TokenPunctuator::BitOr => Operator::BitOr,
                TokenPunctuator::BitXor => Operator::BitXor,
                TokenPunctuator::BitAnd => Operator::BitAnd,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        op
    }
    fn get_precedence(&self, typ: TokenType) -> Precedence {
        match typ {
            TokenType::Punctuator(t) => match &t {
                TokenPunctuator::Plus | TokenPunctuator::Minus => Precedence::Sum,
                TokenPunctuator::Modulo => Precedence::Modulo,
                TokenPunctuator::Multiply | TokenPunctuator::Divide => Precedence::Product,
                TokenPunctuator::Or => Precedence::Or,
                TokenPunctuator::And => Precedence::And,
                TokenPunctuator::Not => Precedence::Prefix,
                TokenPunctuator::LShift | TokenPunctuator::RShift => Precedence::Shift,
                TokenPunctuator::Equal | TokenPunctuator::NE => Precedence::Equality,
                TokenPunctuator::GT
                | TokenPunctuator::GTE
                | TokenPunctuator::LT
                | TokenPunctuator::LTE => Precedence::Comparison,
                TokenPunctuator::BitOr => Precedence::BitOr,
                TokenPunctuator::BitXor => Precedence::BitXor,
                TokenPunctuator::BitAnd => Precedence::BitAnd,
                _ => Precedence::Lowest,
            },
            _ => Precedence::Lowest,
        }
    }
    fn get_precedence_by_operator(&self, typ: &Operator) -> Precedence {
        match typ {
            Operator::Plus | Operator::Subtract => Precedence::Sum,
            Operator::Multiply | Operator::Divide => Precedence::Product,
            Operator::Modulo => Precedence::Modulo,
            _ => unreachable!(),
        }
    }
}

/// 不处理a++等未定义行为
#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Or,         // ||
    And,        // &&
    Equality,   // ==, !=
    Comparison, // <, >, <=, >=
    BitOr,      // |
    BitXor,     // ^
    BitAnd,     // &
    Shift,      // <<, >>
    Sum,        // + -
    Modulo,     // %
    Product,    // * /
    Prefix,     //
}
