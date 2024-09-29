use std::path::StripPrefixError;

use super::{
    expr::{Expr, Operator, Unary, Variable},
    lexer::ILexer,
    program::Program,
    token::{Token, TokenKeyword, TokenPunctuator, TokenType},
};

pub struct Parser {
    lexer: Box<dyn ILexer>,
    current_token: Token,
    peek_token: Token,
    allow_fn_name_empty: bool, //是否允许方法名为空
    allow_return: bool,        //是否允许返回return
    allow_break: bool,         //是否允许break
    allow_continue: bool,      //是否允许continue
}

impl Parser {
    pub fn new(lexer: Box<dyn ILexer>) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::new(TokenType::EOF, 0, 0),
            peek_token: Token::new(TokenType::EOF, 0, 0),
            allow_fn_name_empty: false,
            allow_return: false,
            allow_break: false,
            allow_continue: false,
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
                    let expr = self.parse_call_or_member(None)?;
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
                    let expr = self.parse(true)?;
                    self.next_token(); //)
                    return Ok(expr);
                }
                if matches!(t, TokenPunctuator::LSParen) {
                    self.next_token(); //[
                    let expr = self.parse(true)?;
                    self.next_token(); //]
                    return Ok(expr);
                }
                if matches!(t, TokenPunctuator::Dot) {
                    // self.next_token(); //[
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
                    self.next_token();
                    return Ok(true);
                } else if matches!(ptor, TokenPunctuator::RCParen) {
                    self.next_token();
                    return Ok(true);
                } else if matches!(ptor, TokenPunctuator::RSParen) {
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
                    let expr = Expr::Update(
                        Box::new(Expr::Identifier(ident.clone())),
                        format!("++"),
                        false,
                    );
                    skip_semicolon(self);
                    return Ok(expr);
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
                        return Err(self.err("Unexpected end of input"));
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
                    let expr = self.parse_if_slot();
                    skip_semicolon(self);
                    return expr;
                } else if matches!(t, TokenKeyword::Else) {
                    return Err(self.err("Unexpected token else"));
                } else if matches!(t, TokenKeyword::For) {
                    let expr = self.parse_for_slot();
                    skip_semicolon(self);
                    return expr;
                } else if matches!(t, TokenKeyword::Break) {
                    if !self.allow_break {
                        return Err(self.err("Illegal break statement"));
                    }
                    self.next_token();
                    skip_semicolon(self);
                    return Ok(Expr::Break);
                } else if matches!(t, TokenKeyword::Continue) {
                    if !self.allow_continue {
                        return Err(self.err("Illegal continue statement"));
                    }
                    self.next_token();
                    skip_semicolon(self);
                    return Ok(Expr::Continue);
                } else if matches!(t, TokenKeyword::Return) {
                    if !self.allow_return {
                        return Err(self.err("Illegal return statement"));
                    }
                    self.allow_fn_name_empty = true;
                    self.next_token();
                    if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                        self.next_token();
                        skip_semicolon(self);
                        return Ok(Expr::Return(Box::new(Expr::Empty)));
                    } else {
                        let expr = self.parse(is_skip_semicolon)?;
                        if matches!(
                            expr,
                            Expr::Infix(_, _, _)
                                | Expr::Call(_, _)
                                | Expr::Function(_, _, _)
                                | Expr::Identifier(_)
                                | Expr::Assignment(_, _)
                                | Expr::Assignment2(_)
                                | Expr::Literal(_, _)
                                | Expr::Literal2(_)
                        ) {
                            self.allow_fn_name_empty = false;
                            skip_semicolon(self);
                            skip_semicolon(self);
                            return Ok(Expr::Return(Box::new(expr)));
                        }
                        dbg!(&expr);
                        return Err(self.err("Unexpected token"));
                    }
                } else if matches!(t, TokenKeyword::Function) {
                    let expr = self.parse_function_slot();
                    skip_semicolon(self);
                    return expr;
                } else if matches!(t, TokenKeyword::While) {
                    let expr = self.parse_while_slot();
                    skip_semicolon(self);
                    return expr;
                }
                todo!("{:?}", t);
            }
        };
        Err(self.err("未知解析"))
    }

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
                    // Expr::Return(_) | Expr::Break | Expr::Continue => {
                    //     return Err(self.err("Unexpected token"));
                    // }
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
            "\x1b[31m({}) {} \x1b[39m",
            self.current_token.desc(),
            str,
        );
        panic!("{}", msg);
        return msg;
    }

    fn next_token(&mut self) -> Token {
        let token = self.current_token.clone();
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
        token
    }

    fn parse_body(&mut self, allow_single: bool) -> Result<Expr, String> {
        if !allow_single && !self.current_token.is_ptor(TokenPunctuator::LCParen) {
            return Err(self.err("Unexpected token"));
        }
        if self.current_token.is_ptor(TokenPunctuator::LCParen) {
            self.next_token(); // {
            let mut v = Vec::new();
            loop {
                if self.current_token.is_eof(true) {
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
            self.next_token(); //}
            return Ok(Expr::BlockStatement(v));
        } else {
            let expr = self.parse(false)?;
            if matches!(
                expr,
                Expr::Assignment2(_) | Expr::Call(_, _) | Expr::Return(_)
            ) {
                return Ok(expr);
            } else {
                dbg!(&expr);
                return Err(self.err("Use of future reserved word in strict mode"));
            }
        }
    }

    fn is_valid(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Unary(_, _)
            | Expr::Identifier(_)
            | Expr::TemplateLiteral(_, _)
            | Expr::Literal(_, _)
            | Expr::Literal2(_)
            | Expr::Call(_, _)
            | Expr::Member(_, _)
            | Expr::Sequence(_)
            | Expr::Infix(_, _, _)
            | Expr::Update(_, _, _)
            | Expr::Assignment(_, _)
            | Expr::Assignment2(_)
            | Expr::Variable2(_)
            | Expr::Variable(_, _, _) => true,
            _ => false,
        }
    }
    fn parse_while_slot(&mut self) -> Result<Expr, String> {
        self.next_token(); // while
        if !self.current_token.is_ptor(TokenPunctuator::LParen) {
            return Err(self.err("Unexpected token"));
        }
        self.next_token();
        let test = self.parse(false)?;
        if !self.is_valid(&test){
            return Err(self.err("Unexpected token"));
        }
        if !self.current_token.is_ptor(TokenPunctuator::RParen) {
            return Err(self.err("Unexpected token"));
        }
        self.next_token();
        self.allow_break = true;
        self.allow_continue = true;
        let body = self.parse_body(true)?;
        self.allow_break = false;
        self.allow_continue = false;
        dbg!(&test);
        dbg!(&body);
        Ok(Expr::While(Box::new(test), Box::new(body)))
    }
    fn parse_function_slot(&mut self) -> Result<Expr, String> {
        self.next_token(); // function

        let mut ident: Option<Token> = None;
        if self.allow_fn_name_empty {
            //return funciton(){} 这种情况下可以没有名称
            if self.current_token.is_ident() {
                ident = Some(self.next_token()); //ident
            }
        } else {
            if !self.current_token.is_ident() {
                return Err(self.err("Unexpected token"));
            }
            ident = Some(self.next_token()); //ident
        }
        //

        if !self.current_token.is_ptor(TokenPunctuator::LParen) {
            return Err(self.err("Unexpected token"));
        }
        self.next_token(); //(
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
        self.allow_return = true;
        let body = self.parse_body(false)?;
        self.allow_return = false;
        if let Some(tk) = ident {
            Ok(Expr::Function(
                Box::new(Expr::Identifier(tk.raw.clone())),
                args,
                Box::new(body),
            ))
        } else {
            Ok(Expr::Function(Box::new(Expr::Empty), args, Box::new(body)))
        }
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
            if !self.is_valid(&binary){
                return Err(self.err("Unexpected token"));
            }
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
        self.allow_break = true;
        self.allow_continue = true;
        let body = self.parse_body(true)?;
        self.allow_break = false;
        self.allow_continue = false;

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
        if !self.is_valid(&binary){
            return Err(self.err("Unexpected token"));
        }
        if !self.current_token.is_ptor(TokenPunctuator::RParen) {
            return Err(self.err("Unexpected token"));
        }
        self.next_token(); //)
        let line = self.current_token.line;
        let left_expr = self.parse_body(true)?;

        let mut flag = false;
        if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
            self.next_token();
            flag = true;
        }
        let mut right_expr = Expr::Empty;
        if self.current_token.is_keyword(TokenKeyword::Else) {
            if self.current_token.line == line && !flag {
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
        if self.current_token.is_ptor(TokenPunctuator::RParen) {
            self.next_token(); //)
            return Ok(Expr::Call(Box::new(callee.clone()), Vec::new()));
        }

        let mut v = Vec::new();
        let expr = self.parse(true)?;
        v.push(expr);
        while self.current_token.is_ptor(TokenPunctuator::Comma) {
            self.next_token();
            let expr = self.parse(true)?;
            if !self.is_valid(&expr){
                return Err(self.err("Unexpected token"));
            }
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
            let expr = self.parse(true)?;
            if !self.is_valid(&expr){
                return Err(self.err("Unexpected token"));
            }
            let mut v = Vec::new();
            v.push(expr);
            while self.current_token.is_ptor(TokenPunctuator::Comma) {
                self.next_token();
                let e = self.parse(true)?;
                if !self.is_valid(&e){
                    return Err(self.err("Unexpected token"));
                }
                v.push(e);
            }
            if v.len() == 1 {
                if let Expr::Member(_, property) = mem {
                    *property = Box::new(v.get(0).unwrap().clone());
                } else {
                    return Err(self.err("未解析或异常"));
                }
            } else {
                if let Expr::Member(_, property) = mem {
                    *property = Box::new(Expr::Sequence(v));
                } else {
                    return Err(self.err("未解析或异常"));
                }
            }
            if !self.current_token.is_ptor(TokenPunctuator::RSParen) {
                return Err(self.err("Unexpected end of input"));
            }
            self.next_token(); //]
        }
        Ok(())
    }
    /// 这里还要处理多级 如: a()[1]  a[1]()    a[1]()[1]()...
    fn parse_call_or_member(&mut self, prefix: Option<Expr>) -> Result<Expr, String> {
        let mut id = Expr::Empty;
        if prefix == None {
            let token = self.next_token();
            id = Expr::Identifier(token.raw.clone())
        } else {
            id = prefix.unwrap();
        }
        let mut expr = Expr::Empty;
        loop {
            if self.current_token.is_eof(true) {
                break;
            }
            if self.current_token.is_ptor(TokenPunctuator::RCParen) {
                break;
            } else if self.current_token.is_ptor(TokenPunctuator::RParen) {
                break;
            }
            if self.current_token.is_ptor(TokenPunctuator::LParen) {
                //(
                if Expr::Empty == expr {
                    expr = id.clone();
                } else {
                    // let line = self.current_token.line;
                    expr = self.parse_call_slot(&mut expr)?;

                    // if self.current_token.line == line {
                    //     if self.current_token.checked_keyword(){
                    //         break;
                    //     }
                    //    //println!("{:?}",self.current_token);
                    // }
                    // break;
                }
            } else if self.current_token.is_ptor(TokenPunctuator::LSParen)
                || self.current_token.is_ptor(TokenPunctuator::Dot)
            {
                //[
                if Expr::Empty == expr {
                    expr = Expr::Member(Box::new(id.clone()), Box::new(Expr::Empty));
                } else {
                    expr = Expr::Member(Box::new(expr), Box::new(Expr::Empty));
                }
                let _ = self.parse_member_slot(&mut expr);
            } else {
                break;
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
