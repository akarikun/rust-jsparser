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

    fn parse_var_slot(&mut self, typ: String) -> Vec<Option<Expr>> {
        let mut v: Vec<Option<Expr>> = Vec::new();
        self.next_token(); //skip  'var' 'let' 'const'
        let name = match &self.current_token.typ {
            TokenType::Ident(name) => name.clone(),
            _ => panic!("{}", self.err("脚本异常")),
        };
        while !self.current_token.is_eof(true) {
            let expr = self.parse_expression(1)[0].clone();
            if let Expr::Assignment(ident, exp) = expr {
                v.push(Some(Expr::Variable(typ.clone(), ident, exp)));
            } else if let Expr::Identifier(ident) = expr {
                v.push(Some(Expr::Variable(
                    typ.clone(),
                    ident,
                    Box::new(Expr::Empty),
                )));
            }
            if self.current_token.is_ptor(TokenPunctuator::Comma) {
                self.next_token(); //skip ','
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
            if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
            }
            if self.current_token.is_eof(false) {
                break;
            }
            match &self.current_token.typ {
                TokenType::Ident(_) | TokenType::Number(_) => {
                    v.push(self.parse_expression(1).pop().unwrap());
                }
                TokenType::Keyword(t) => {
                    let k = t.to_raw();
                    match &t {
                        TokenKeyword::Let | TokenKeyword::Var | TokenKeyword::Const => {
                            for i in self.parse_var_slot(k.to_string()) {
                                v.push(i.unwrap());
                            }
                        }
                        TokenKeyword::If => {
                            v.push(self.parse_if_slot());
                        }
                        TokenKeyword::Return => todo!(),
                        _ => todo!("{:?}", t),
                    }
                }
                TokenType::Punctuator(t) => match &t {
                    TokenPunctuator::Semicolon => v.push(Expr::Empty),
                    TokenPunctuator::Comma => {
                        self.next_token();
                        continue;
                    }
                    TokenPunctuator::LParen | TokenPunctuator::Not => {
                        v.push(self.parse_expression(1).pop().unwrap());
                    }
                    _ => todo!("{:?}", t),
                },
                _ => todo!("{:?}", self.current_token.typ),
            }
        }
        v
    }
    fn check_diff_line(&self) -> bool {
        return self.current_token.line != self.peek_token.line;
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
            }
            match &self.current_token.typ {
                TokenType::Number(t) | TokenType::Ident(t) => {
                    let ident = t.clone();
                    match &self.peek_token.typ {
                        TokenType::EOF => {
                            self.next_token(); //ident
                            v.push(Expr::Identifier(ident));
                        }
                        TokenType::Ident(t2) => {
                            if self.check_diff_line() {
                                self.next_token(); //ident
                                v.push(Expr::Identifier(ident));
                                break;
                            } else {
                                panic!("{:?}", self.err("脚本异常"));
                            }
                        }
                        TokenType::Punctuator(t2) => {
                            if t2.is_precedence() {
                                v.push(self.parse_base_expression(Precedence::Lowest));
                            } else {
                                match &t2 {
                                    TokenPunctuator::INC => {
                                        let expr =
                                            self.parse_update_slot(ident, Operator::INC, false);
                                        v.push(expr);
                                    }
                                    TokenPunctuator::LParen
                                    | TokenPunctuator::LSParen
                                    | TokenPunctuator::Dot => {
                                        let expr = self.parse_base_expression(Precedence::Lowest);
                                        v.push(expr);
                                    }
                                    TokenPunctuator::Comma => v.push(Expr::Identifier(ident)),
                                    TokenPunctuator::Semicolon => {
                                        self.next_token(); //ident
                                        v.push(Expr::Identifier(ident))
                                    }
                                    TokenPunctuator::MOV => {
                                        self.next_token(); //ident
                                        self.next_token(); //=
                                        let expr = self.parse_base_expression(Precedence::Lowest);
                                        v.push(Expr::Assignment(ident, Box::new(expr)));
                                    }
                                    _ => todo!("{:?}", t2),
                                }
                            }
                        }
                        _ => todo!("{:?}", self.peek_token.typ),
                    }
                }
                TokenType::Punctuator(t) => match &t {
                    TokenPunctuator::LParen
                    | TokenPunctuator::Not
                    | TokenPunctuator::Minus
                    | TokenPunctuator::Plus => {
                        v.push(self.parse_base_expression(Precedence::Lowest));
                    }
                    _ => {
                        println!("{:?}", v);
                        self.log();
                        todo!("{:?}", &t)
                    }
                },
                _ => todo!("{:?}", self.current_token.typ),
            }
            self.next_token();
        }
        v
    }
    ///创建新的解析,括号中的表达式使用全量扫描
    fn new_parser(&self, v: Vec<Token>, count: usize, is_checked: bool) -> (Vec<Expr>, Parser) {
        let list: Rc<RefCell<Vec<Token>>> = Rc::new(RefCell::new(v));
        let mut parser = Parser::new(Box::new(TokenList::new(Rc::clone(&list))));
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
                                Expr::Prefix(prefix, Box::new(self.parse_member_slot(ident)))
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
            _ => todo!(),
        };
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
        let expr = Expr::Update(ident, op, prefix);
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

    fn parse_else_slot(&mut self) -> Expr {
        self.next_token(); //else
        if self.current_token.is_keyword(TokenKeyword::If){
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
        } 
        else {
            return  Expr::Expression(Box::new(self.parse_expression(1)[0].clone()));
        }
    }
    fn parse_if_slot(&mut self) -> Expr {
        self.next_token(); //skip if
        if self.current_token.is_ptor(TokenPunctuator::LParen) {
            self.next_token(); //skip '('
            let list: Vec<Token> =
                self.get_token_duration(TokenPunctuator::LParen, TokenPunctuator::RParen);
            let (condition, _) = self.new_parser(list, 1, true);

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
                    println!("{:?}",expr);
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
            if !self.current_token.is_eof(false){
                if self.current_token.line == line {
                    panic!("{:?}", self.err("Unexpected token else"));
                }
                else{
                    if self.current_token.is_keyword(TokenKeyword::Else) {
                        expr2 = self.parse_else_slot();
                    }
                }
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
            return Expr::Call(ident, Vec::new());
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
        Expr::Call(ident, args)
    }
    //a[   a.
    fn parse_member_slot(&mut self, ident: String) -> Expr {
        self.next_token(); //ident
        let k = self.current_token.clone();
        let mut args = Vec::new();
        if k.is_ptor(TokenPunctuator::LSParen) {
            self.next_token(); //[
            let list = self.get_token_duration(TokenPunctuator::LSParen, TokenPunctuator::RSParen);
            let (expr, mut parser) = self.new_parser(list, 1, false);
            args.push(expr[0].clone());
            while parser.current_token.is_ptor(TokenPunctuator::Comma) {
                parser.next_token(); //skip ','
                let exprs = parser.parse_base_expression(Precedence::Lowest);
                args.push(exprs.clone());
            }
            Expr::Member(ident, args)
        } else if k.is_ptor(TokenPunctuator::Dot) {
            self.next_token(); //.
            if self.current_token.is_ident() {
                args.push(Expr::Identifier(self.current_token.raw.to_string()));
                Expr::Member(ident, args)
            } else {
                panic!("脚本异常")
            }
        } else {
            panic!()
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
    Product,    // * /
    Prefix,     //
}
