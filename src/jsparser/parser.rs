use std::{cell::RefCell, rc::Rc};

use super::{
    expr::{Expr, Operator, Prefix, Program},
    lexer::{ILexer, TokenList},
    token::{Token, TokenKeyword, TokenPunctuator, TokenType},
};

pub struct Parser {
    lexer: Box<dyn ILexer>,
    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(lexer: Box<dyn ILexer>) -> Self {
        Self::_new(lexer, TokenKeyword::None)
    }

    fn _new(lexer: Box<dyn ILexer>, key: TokenKeyword) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::new(TokenType::EOF, 0, 0),
            peek_token: Token::new(TokenType::EOF, 0, 0),
        };
        parser.next_token();
        parser.next_token();
        parser
    }

    fn log(&self) {
        crate::println(
            31,
            "log",
            format!(
                "<{}>,\n     <{}>",
                &self.current_token.desc(),
                &self.peek_token.desc()
            ),
        );
    }

    fn err(&self, str: &str) -> String {
        format!("{},token:{}", str, self.current_token.desc())
    }

    fn next_token(&mut self) -> Token {
        let token = self.current_token.clone();
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
        token
    }
    pub fn parse_program(&mut self) -> Program {
        let mut statements: Vec<Expr> = Vec::new();
        while self.current_token.typ != TokenType::EOF {
            for expr in self.parse_statement() {
                if let Expr::Unexpected(msg) = expr {
                    crate::println(31, "Uncaught SyntaxError: Unexpected token", msg.clone());
                } else {
                    statements.push(expr.clone());
                }
            }
            self.next_token();
        }
        Program {
            statements: statements,
        }
    }

    ///匹配：var a = 1;   a=1;   (最后有;会过滤掉)
    fn parse_var_slot(&mut self) -> Vec<Option<Expr>> {
        let mut v: Vec<Option<Expr>> = Vec::new();

        let mut typ: Option<String> = None;
        if self.current_token.is_keyword(TokenKeyword::Let)
            || self.current_token.is_keyword(TokenKeyword::Var)
            || self.current_token.is_keyword(TokenKeyword::Const)
        {
            typ = Some(self.current_token.raw.clone());
            self.next_token(); //skip  'var' 'let' 'const'
        }

        let mut line =self.current_token.line;
        while !self.current_token.is_eof(false) {
            // if !self.current_token.is_ident() {
            //     panic!("{}", self.err("脚本异常"));
            // }
            let name = self.current_token.raw.clone();
            let expr = self.parse_ident_num(name.clone()).clone();

            if let Some(t) = &typ {
                match &expr {
                    Expr::Identifier(ident) => {
                        v.push(Some(Expr::Variable(
                            t.clone(),
                            ident.clone(),
                            Box::new(Expr::Empty),
                        )));
                    }
                    Expr::Assignment(ident, exp) => {
                        if let Some(t) = &typ {
                            v.push(Some(Expr::Variable(t.clone(), ident.clone(), exp.clone())));
                        }
                    }
                    Expr::Call(_, _) | Expr::Infix(_, _, _) | Expr::Update(_, _, _) => v.push(Some(expr)),
                    _ => {
                        panic!("{:?}", &expr);
                    }
                };
            } else {
                match &expr {
                    Expr::Identifier(_) => {
                        v.push(Some(Expr::Expression(Box::new(Expr::Empty))));
                    }
                    Expr::Infix(_, _, _) | Expr::Update(_, _, _) => v.push(Some(expr)),
                    _ => {
                        println!("{:?}",&expr);
                        v.push(Some(Expr::Expression(Box::new(expr))));
                    }
                };
            }

            if self.current_token.is_ptor(TokenPunctuator::Comma) {
                self.next_token(); //skip ','
            } else if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
                break;
            } else {
                break;
            }
        }
        v
    }

    ///第一层  step: 1->2
    fn parse_statement(&mut self) -> Vec<Expr> {
        let mut v: Vec<Expr> = Vec::new();
        loop {
            if self.current_token.is_eof(false) {
                break;
            }
            match &self.current_token.typ {
                TokenType::Keyword(t) => {
                    let k = t.to_raw();
                    match &t {
                        TokenKeyword::Let | TokenKeyword::Var | TokenKeyword::Const => {
                            for i in self.parse_var_slot() {
                                v.push(i.unwrap());
                            }
                            continue;
                        }
                        TokenKeyword::If => {
                            v.push(self.parse_if_slot());
                        }
                        TokenKeyword::For => {
                            v.push(self.parse_for_slot());
                        }
                        TokenKeyword::Return => todo!(),
                        _ => todo!("{:?}", t),
                    }
                }
                _ => {
                    for i in self.parse_expression(0) {
                        v.push(i);
                    }
                }
            }
        }
        v
    }

    fn parse_ident_num(&mut self, ident: String) -> Expr {
        let is_ident = self.current_token.is_ident();
        let token = self.peek_token.clone();
        let current_token = self.current_token.clone();

        let action = |p: &mut Self| p.parse_base_expression(Precedence::Lowest);

        let ident_num_action = || {
            if is_ident {
                Expr::Identifier(ident.clone())
            } else {
                Expr::Number(ident.clone().parse::<f64>().unwrap())
            }
        };

        match token.typ {
            TokenType::EOF => {
                self.next_token(); //ident
                ident_num_action()
            }
            TokenType::Ident(t2) => {
                if current_token.line != token.line {
                    self.next_token(); //ident
                    ident_num_action()
                } else {
                    panic!("{:?}", self.err("脚本异常"));
                }
            }
            TokenType::Punctuator(t2) => {
                if t2.is_precedence() {
                    let expr = action(self);
                    self.next_token();
                    return expr;
                } else {
                    match &t2 {
                        TokenPunctuator::INC => self.parse_update_slot(ident, Operator::INC, false),
                        TokenPunctuator::LParen
                        | TokenPunctuator::LSParen
                        | TokenPunctuator::Dot => {
                            let expr = action(self);
                            return expr;
                        },
                        TokenPunctuator::Comma => ident_num_action(),
                        TokenPunctuator::Semicolon => {
                            self.next_token(); //ident
                            let expr = ident_num_action();
                            expr
                        }
                        TokenPunctuator::MOV => {
                            self.next_token(); //ident
                            self.next_token(); //=
                            let expr = Expr::Assignment(ident, Box::new(action(self)));
                            self.next_token();
                            expr
                        }
                        _ => todo!("{:?}", t2),
                    }
                }
            }
            TokenType::Keyword(t2) => {
                match t2 {
                    TokenKeyword::In | TokenKeyword::Of => {
                        self.next_token(); //ident
                        self.next_token(); //in / of
                        let expr = ident_num_action();
                        let op = if t2 == TokenKeyword::In {
                            Operator::In
                        } else {
                            Operator::Of
                        };
                        Expr::Infix(
                            Box::new(Expr::Identifier(ident.clone())),
                            op,
                            Box::new(expr),
                        )
                    }
                    _ => todo!("{:?}", t2),
                }
            }
            _ => todo!("{:?}", token),
        }
    }
    ///第二层 step: 2->2或2->3
    fn parse_expression(&mut self, count: usize) -> Vec<Expr> {
        let mut v = Vec::new();
        loop {
            if count > 0 && v.len() == count {
                return v;
            }
            if self.current_token.is_eof(false) {
                break;
            }
            if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
                v.push(Expr::Empty);
                continue;
            }
            //统一在allow最后判断做扫尾操作，有特殊处理的需要及时处理扫尾
            let mut allow = false;
            match &self.current_token.typ {
                TokenType::Number(t) | TokenType::Ident(t) => {
                    if self.peek_token.is_eof(false) {
                        if self.current_token.is_ident() {
                            v.push(Expr::Identifier(t.clone()))
                        } else {
                            v.push(Expr::Number(t.clone().parse::<f64>().unwrap()))
                        }
                        self.next_token();
                        continue;
                    }
                    if self.peek_token.is_ptor(TokenPunctuator::MOV) {
                        for i in self.parse_var_slot() {
                            v.push(i.unwrap());
                        }
                        continue;
                    } else if self.peek_token.is_ptor(TokenPunctuator::INC) {
                        v.push(self.parse_ident_num(self.peek_token.clone().raw));
                    }
                    else {
                        allow = true;
                    }
                }
                TokenType::Punctuator(t) => {
                    if t.is_precedence() {
                        allow = true;
                    } else if t == &TokenPunctuator::LParen {
                        self.next_token();
                        let list: Vec<Token> = self
                            .get_token_duration(TokenPunctuator::LParen, TokenPunctuator::RParen);
                        let (expr, _) = self.new_parser(list, 1, true);
                        if !self.current_token.is_ptor(TokenPunctuator::RParen) {
                            panic!("{}", self.err("Unexpected end of input"));
                        }
                        self.next_token();
                        v.push(expr[0].clone());
                    } else {
                        todo!("{:?}", t)
                    }
                }
                TokenType::Keyword(t) => match &t {
                    TokenKeyword::In => {
                        allow = true;
                    }
                    _ => {
                        todo!("{:?}", self.current_token.typ)
                    }
                },
                _ => todo!("{:?}", self.current_token.typ),
            }
             
            if allow {
                v.push(self.parse_base_expression(Precedence::Lowest));
                let mut line =self.current_token.line;
                if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                    line = 0;
                }
                if self.peek_token.checked_keyword(){
                    if self.peek_token.line == line {
                        panic!("{}",self.err("Unexpected token"));
                    }
                    self.next_token();
                    return v;
                }
            }
            self.next_token();
            if allow && self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
            }
        }
        v
    }
    ///创建新的解析,括号中的表达式使用全量扫描
    fn new_parser(
        &self,
        v: Vec<Token>,
        // key: TokenKeyword,
        count: usize,
        is_checked: bool,
    ) -> (Vec<Expr>, Parser) {
        let list: Rc<RefCell<Vec<Token>>> = Rc::new(RefCell::new(v));
        let mut parser = Self::_new(
            Box::new(TokenList::new(Rc::clone(&list))),
            TokenKeyword::None,
        );
        if count == 0 {
            let expr = parser.parse_statement();
            if is_checked && !parser.current_token.is_eof(false) {
                panic!("{}", self.err("子解析异常"))
            }
            return (expr, parser);
        } else {
            let expr = parser.parse_expression(count);
            if is_checked && !parser.current_token.is_eof(false) {
                panic!("{}", self.err("子解析异常"))
            }
            return (expr, parser);
        }
    }

    fn get_prefix(&self, tk: Token) -> Prefix {
        match tk.typ {
            TokenType::Punctuator(t) => match t {
                TokenPunctuator::Not => Prefix::Not,
                TokenPunctuator::Plus => Prefix::Abs,
                TokenPunctuator::Minus => Prefix::Negate,
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
    ///第三层(base) 转换至Expr step: 3->3
    fn parse_base_expression(&mut self, infix: Precedence) -> Expr {
        let mut left: Expr = Expr::Empty;
        left = match &self.current_token.typ {
            TokenType::Ident(t) => {
                let ident = t.clone();
                if self.peek_token.is_ptor(TokenPunctuator::LParen) {
                    // a(
                    self.parse_call_slot(ident)
                } else if self.peek_token.is_ptor(TokenPunctuator::LSParen)
                    || self.peek_token.is_ptor(TokenPunctuator::Dot)
                {
                    //a[   a.
                    self.parse_member_slot(ident)
                } else {
                    Expr::Identifier(ident.clone())
                }
            }
            TokenType::Number(num) => Expr::Number(num.parse().unwrap()),
            TokenType::Punctuator(t) => {
                if t.is_prefix() {
                    //处理 -a !a +a的情况
                    let tk = self.next_token(); //skip prefix
                    let prefix = self.get_prefix(tk);
                    match &self.current_token.typ {
                        TokenType::Number(num) => {
                            Expr::Prefix(prefix, Box::new(Expr::Number(num.parse().unwrap())))
                        }
                        TokenType::Ident(t) => {
                            let ident = t.clone();
                            if self.peek_token.is_ptor(TokenPunctuator::LParen) {
                                //a(
                                Expr::Prefix(prefix, Box::new(self.parse_call_slot(ident)))
                            } else if self.peek_token.is_ptor(TokenPunctuator::LSParen)
                                || self.peek_token.is_ptor(TokenPunctuator::Dot)
                            {
                                //a[   a.
                                //Expr::Prefix(prefix, Box::new(self.parse_member_slot(ident)))
                                self.parse_member_slot(ident)
                            } else {
                                Expr::Prefix(prefix, Box::new(Expr::Identifier(ident.clone())))
                            }
                        }
                        _ => panic!(),
                    }
                } else {
                    match &t {
                        TokenPunctuator::Semicolon => Expr::Empty,
                        TokenPunctuator::LParen => {
                            self.next_token();
                            let list: Vec<Token> = self.get_token_duration(
                                TokenPunctuator::LParen,
                                TokenPunctuator::RParen,
                            );
                            let (expr, _) = self.new_parser(list, 1, true);
                            expr[0].clone()
                        }
                        _ => todo!("{:?}", t),
                    }
                }
            }
            _ => {
                todo!("{:?}", &self.current_token.typ)
            }
        };

        //在这里判断下 peek_token
        if self.peek_token.is_eof(true) {
            self.next_token();
            return left;
        }
        if self.peek_token.checked_keyword(){
            // self.next_token();
            return left;
        }
        if self.peek_token.is_ptor(TokenPunctuator::Comma){
            self.next_token();
            return left;
        }
        while infix < self.get_precedence(self.peek_token.typ.clone()) {
            left = match &self.peek_token.typ {
                TokenType::Punctuator(t) => {
                    if t.is_precedence() {
                        self.next_token();
                        left = self.parse_base_infix(left);
                        left
                    } else {
                        todo!()
                    }
                    // _ => todo!("{:?}", t),
                }
                _ => todo!(),
            }
        }
        left
    }
    fn get_precedence(&self, typ: TokenType) -> Precedence {
        match typ {
            TokenType::Punctuator(t) => match &t {
                TokenPunctuator::Plus | TokenPunctuator::Minus => Precedence::Sum,
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
    fn parse_base_infix(&mut self, left: Expr) -> Expr {
        let precedence = self.get_precedence(self.current_token.typ.clone());
        let op = match &self.current_token.typ {
            TokenType::Punctuator(t) => match t {
                TokenPunctuator::Plus => Operator::Plus,
                TokenPunctuator::Minus => Operator::Minus,
                TokenPunctuator::Multiply => Operator::Multiply,
                TokenPunctuator::Divide => Operator::Divide,
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
        self.next_token(); // Skip operator
        let right = self.parse_base_expression(precedence);
        Expr::Infix(Box::new(left), op, Box::new(right))
    }
    //a++,a--,++a,--a
    fn parse_update_slot(&mut self, ident: String, op: Operator, prefix: bool) -> Expr {
        let expr = Expr::Update(Box::new(Expr::Identifier(ident)), op, prefix);
        self.next_token(); //skip ident
        self.next_token(); //skip ++
                           // self.skip_next_token_ptor(TokenPunctuator::Semicolon, true); //skip ;
        expr
    }

    ///读取token片断,注意current_token不能是 '(' , '[' , '{' ,在调用时请先处理
    fn get_token_duration(&mut self, left: TokenPunctuator, right: TokenPunctuator) -> Vec<Token> {
        let mut list: Vec<Token> = Vec::new();
        let mut paren = 1;
        let mut tk = self.current_token.clone();
        while tk.typ != TokenType::EOF {
            if tk.is_ptor(left.clone()) {
                paren += 1;
            } else if tk.is_ptor(right.clone()) {
                paren -= 1;
                if paren == 0 {
                    break;
                }
            }
            list.push(tk);
            self.next_token();
            tk = self.current_token.clone()
        }
        list
    }

    fn parse_for_body(&mut self) -> Expr {
        if self.current_token.is_ptor(TokenPunctuator::LCParen) {
            self.next_token(); // {

            let list2: Vec<Token> =
                self.get_token_duration(TokenPunctuator::LCParen, TokenPunctuator::RCParen);
            let (body, _) = self.new_parser(list2, 0, true);

            if !self.current_token.is_ptor(TokenPunctuator::RCParen) {
                panic!("{}", self.err("Unexpected end of input"));
            }
            self.next_token(); // }
            Expr::BlockStatement(body)
        } else {
            let body = self.parse_expression(1);
            if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
            }
            Expr::Expression(Box::new(body[0].clone()))
        }
    }

    fn parse_for_slot(&mut self) -> Expr {
        self.next_token(); //for

        let mut filter = |p: &mut Self| {
            if p.current_token.is_ptor(TokenPunctuator::Semicolon) {
                p.next_token();
            }
        };

        if !&self.current_token.is_ptor(TokenPunctuator::LParen) {
            panic!("{}", self.err("Unexpected end of input"));
        }
        self.next_token(); // (
        let list: Vec<Token> =
            self.get_token_duration(TokenPunctuator::LParen, TokenPunctuator::RParen);

        let mut count = 0;
        let mut IN = 0;
        let mut OF = 0;
        for i in &list {
            if i.is_ptor(TokenPunctuator::Semicolon) {
                count += 1;
            } else if i.is_keyword(TokenKeyword::In) {
                IN += 1;
            } else if i.is_keyword(TokenKeyword::Of) {
                OF += 1;
            }
        }

        if !&self.current_token.is_ptor(TokenPunctuator::RParen) {
            panic!("{}", self.err("Unexpected end of input"));
        }
        self.next_token(); // )

        if !((count == 2 && IN + OF == 0) || (count == 0 && IN + OF == 1)) {
            panic!("Unexpected end of input");
        }
        if IN + OF == 1 {
            let (expr, _) = self.new_parser(list, 0, true);
            if IN == 1 {
                let body = self.parse_for_body();
                filter(self);
                Expr::ForIn(Box::new(expr[0].clone()), Box::new(body))
            } else {
                let body = self.parse_for_body();
                filter(self);
                Expr::ForOf(Box::new(expr[0].clone()), Box::new(body))
            }
        } else if count == 2 {
            let (expr, _) = self.new_parser(list, 0, false);
            let mut expr = expr;

            if expr.len() == 2 {
                expr.push(Expr::Empty);
            }
            let body = self.parse_for_body();
            filter(self);
            Expr::For(
                Box::new(expr[0].clone()),
                Box::new(expr[1].clone()),
                Box::new(expr[2].clone()),
                Box::new(body),
            )
        } else {
            panic!("{}", self.err("Unexpected end of input"));
        }
    }
    fn parse_else(&mut self) -> Expr {
        self.next_token(); //else
        if self.current_token.is_keyword(TokenKeyword::If) {
            return self.parse_if_slot();
        }
        if self.current_token.is_ptor(TokenPunctuator::LCParen) {
            self.next_token();
            let list: Vec<Token> =
                self.get_token_duration(TokenPunctuator::LCParen, TokenPunctuator::RCParen);
            let (expr, _) = self.new_parser(list, 0, true);
            if self.current_token.is_ptor(TokenPunctuator::RCParen) {
                self.next_token();
                return Expr::BlockStatement(expr);
            } else {
                panic!("{}", self.err("Unexpected end of input"));
            }
        } else {
            return Expr::Expression(Box::new(
                self.parse_expression(1)
                    .first()
                    .expect("Unexpected end of input")
                    .clone(),
            ));
        }
    }
    fn parse_if_slot(&mut self) -> Expr {
        self.next_token(); //skip if

        if self.current_token.is_ptor(TokenPunctuator::LParen) {
            self.next_token(); //skip '('
            let list: Vec<Token> =
                self.get_token_duration(TokenPunctuator::LParen, TokenPunctuator::RParen);
            let (condition, _) = self.new_parser(list, 1, true);
            // println!("{:?}", condition);
            self.next_token(); //skip ')'
            let mut expr1 = Expr::Empty;
            let mut expr2 = Expr::Empty;
            let mut line = 0;

            if self.current_token.is_ptor(TokenPunctuator::LCParen) {
                self.next_token();
                let list: Vec<Token> =
                    self.get_token_duration(TokenPunctuator::LCParen, TokenPunctuator::RCParen);
                let (expr, _) = self.new_parser(list, 0, true);
                if self.current_token.is_ptor(TokenPunctuator::RCParen) {
                    self.next_token();
                    expr1 = Expr::BlockStatement(expr);
                } else {
                    panic!("{}", self.err("Unexpected end of input"));
                }
            } else {
                line = self.current_token.line;
                let expr = self.parse_expression(1);
                if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                    line = 0;
                    self.next_token();
                }
                expr1 = Expr::Expression(Box::new(expr[0].clone()));
            }
            if self.current_token.is_keyword(TokenKeyword::Else) {
                expr2 = self.parse_else();
            }
            return Expr::If(
                Box::new(condition[0].clone()),
                Box::new(expr1),
                Box::new(expr2),
            );
        }
        panic!("Unexpected end of input")
    }
    //a(
    fn parse_call_slot(&mut self, ident: String) -> Expr {
        self.next_token(); //ident
        self.next_token(); //(
        if self.current_token.is_ptor(TokenPunctuator::RParen) {
            return Expr::Call(Box::new(Expr::Identifier(ident)), Vec::new());
        }

        let list = self.get_token_duration(TokenPunctuator::LParen, TokenPunctuator::RParen);
        let mut args: Vec<Expr> = Vec::new();
        let (expr, mut parser) = self.new_parser(list, 1, false);
        args.push(expr[0].clone());
        while parser.current_token.is_ptor(TokenPunctuator::Comma) {
            parser.next_token(); //skip ','
            let exprs = parser.parse_base_expression(Precedence::Lowest);
            args.push(exprs.clone());
        }
        if !self.current_token.is_ptor(TokenPunctuator::RParen){
            panic!("{}",self.err("Unexpected end of input"))
        }
        Expr::Call(Box::new(Expr::Identifier(ident)), args)
    }
    //a[   a.
    fn parse_member_slot(&mut self, ident: String) -> Expr {
        // let k = self.peek_token.clone();
        if self.peek_token.is_ptor(TokenPunctuator::Dot) {
            self.next_token(); //ident
            while self.current_token.is_ptor(TokenPunctuator::Dot) {
                //a.b
                self.next_token(); //skipt '.'
                if self.current_token.is_ident() {
                    let member = self.current_token.raw.clone();
                    let expr = self.parse_member_slot(member);
                    let mem = Expr::Member(Box::new(Expr::Identifier(ident)), Box::new(expr));
                    return mem;
                }
            }
            panic!("{:?}", self.err("Unexpected token"));
        } else if self.peek_token.is_ptor(TokenPunctuator::LSParen) {
            self.next_token(); //ident

            let mut mem = Expr::Identifier(ident.clone());
            while self.current_token.is_ptor(TokenPunctuator::LSParen) {
                let mut args = Vec::new();
                //a[
                self.next_token(); //[
                let list =
                    self.get_token_duration(TokenPunctuator::LSParen, TokenPunctuator::RSParen);
                let (expr, mut parser) = self.new_parser(list, 0, false);
                for i in expr{
                    args.push(i);
                }
                while parser.peek_token.is_ptor(TokenPunctuator::Comma) {
                    parser.next_token(); //skip ','
                    let exprs = parser.parse_base_expression(Precedence::Lowest);
                    args.push(exprs);
                }
                if args.len() > 1 {
                    mem = Expr::Member(Box::new(mem), Box::new(Expr::Sequence(args)));
                } else {
                    mem = Expr::Member(Box::new(mem), Box::new(args[0].clone()));
                }
                if self.peek_token.is_ptor(TokenPunctuator::LSParen){
                    if self.current_token.is_ptor(TokenPunctuator::RSParen){
                        self.next_token();
                    }
                }
            }
            if !self.current_token.is_ptor(TokenPunctuator::RSParen){
                panic!("{}",self.err("Unexpected end of input"))
            }
            return mem;
        } else {
            Expr::Identifier(ident)
        }
    }
    //这里还要处理多级 如: a()[1]  a[1]()    a[1]()[1]()...
    fn parse_call_or_member()->Expr{
        Expr::Empty
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
    Product,    // * /
    Prefix,     //
}
