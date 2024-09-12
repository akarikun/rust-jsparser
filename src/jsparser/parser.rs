use std::{cell::RefCell, rc::Rc};

use super::{
    expr::{Expr, Operator, Prefix, Program, Stmt},
    lexer::{ILexer, TokenList},
    token::{Token, TokenPunctuator, TokenType},
};

pub struct Parser<'a> {
    lexer: Box<dyn ILexer + 'a>,
    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
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
        let mut statements: Vec<Stmt> = Vec::new();
        while self.current_token.typ != TokenType::EOF {
            for i in self.parse_statement() {
                if let Some(stmt) = i {
                    if let Stmt::Unexpected(msg) = stmt {
                        crate::println(31, "Uncaught SyntaxError: Unexpected token", msg.clone());
                    } else {
                        statements.push(stmt.clone());
                    }
                } else {
                    //crate::println(33, "Uncaught SyntaxError: Unexpected token", "".to_string());
                }
            }
            self.next_token();
        }
        Program {
            statements: statements,
        }
    }

    fn skip_next_token_ptor(&mut self, typ: TokenPunctuator, is_skip: bool) -> bool {
        match &self.peek_token.typ {
            TokenType::Punctuator(t) => {
                if typ == *t {
                    if is_skip {
                        self.next_token();
                    }
                    return true;
                }
                return false;
            }
            _ => return false,
        }
    }
    // fn skip_next_token(&mut self, typ: TokenType, is_skip: bool) -> bool {
    //     if typ == self.peek_token.typ {
    //         if is_skip {
    //             self.next_token();
    //         }
    //         return true;
    //     }
    //     false
    // }
    ///第一层 转换至Stmt  step: 1->2
    fn parse_statement(&mut self) -> Vec<Option<Stmt>> {
        let mut v = Vec::new();
        loop {
            if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
            }
            if self.current_token.is_eof(false) {
                break;
            }
            match &self.current_token.typ {
                TokenType::Ident(t) => {
                    if self.peek_token.is_ptor(TokenPunctuator::MOV) {
                        v.push(Some(Stmt::Assignment(
                            t.to_string(),
                            self.parse_expression(1).pop().unwrap().unwrap(),
                        )));
                    } else {
                        v.push(Some(Stmt::Expression(
                            self.parse_expression(1).pop().unwrap().unwrap(),
                        )));
                    }
                }
                TokenType::Number(_) => {
                    v.push(Some(Stmt::Expression(
                        self.parse_expression(1).pop().unwrap().unwrap(),
                    )));
                }
                TokenType::Keyword(t) => {
                    let k = t.to_raw();
                    self.next_token(); //'let'
                    let name = match &self.current_token.typ {
                        TokenType::Ident(name) => name.clone(),
                        _ => panic!("{}", self.err("脚本异常")),
                    };
                    if !self.skip_next_token_ptor(TokenPunctuator::MOV, true) {
                        panic!("脚本异常");
                        // v.push(Some(Stmt::Unexpected(self.peek_token.desc())));
                    }
                    self.next_token(); //ident
                    let expr = self.parse_expression(1)[0].clone().expect("脚本异常");
                    v.push(Some(Stmt::Variable(format!("{}", k), name, expr)));
                }
                TokenType::Punctuator(t) => match &t {
                    TokenPunctuator::Semicolon => v.push(None),
                    TokenPunctuator::LParen => {
                        v.push(Some(Stmt::Expression(
                            self.parse_expression(1).pop().unwrap().unwrap(),
                        )));
                    }
                    TokenPunctuator::Comma => {
                        self.next_token();
                        continue;
                    }
                    TokenPunctuator::Not => {
                        v.push(Some(Stmt::Expression(
                            self.parse_expression(1).pop().unwrap().unwrap(),
                        )));
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

    ///第二层 转换至Expr step: 2->2或2->3
    fn parse_expression(&mut self, count: usize) -> Vec<Option<Expr>> {
        let mut v = Vec::new();
        loop {
            if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
            }
            if self.current_token.is_eof(false) {
                break;
            }
            if count > 0 && v.len() == count {
                return v;
            }
            match &self.current_token.typ {
                TokenType::Number(t) | TokenType::Ident(t) => {
                    let ident = t.clone();
                    match &self.peek_token.typ {
                        TokenType::EOF => {
                            self.next_token(); //ident
                            v.push(Some(Expr::Identifier(ident)));
                        }
                        TokenType::Ident(t2) => {
                            if self.check_diff_line() {
                                self.next_token(); //ident
                                v.push(Some(Expr::Identifier(ident)));
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
                                    TokenPunctuator::LParen => {
                                        let expr = self.parse_base_expression(Precedence::Lowest);
                                        v.push(expr);
                                    }
                                    TokenPunctuator::Comma => {
                                        self.next_token(); //ident
                                        v.push(Some(Expr::Identifier(ident)))
                                    }
                                    TokenPunctuator::Semicolon => {
                                        self.next_token(); //ident
                                        v.push(Some(Expr::Identifier(ident)))
                                    }
                                    TokenPunctuator::MOV => {
                                        self.next_token(); //ident
                                        self.next_token(); //=
                                        let expr = self.parse_base_expression(Precedence::Lowest);
                                        v.push(expr);
                                    }
                                    _ => todo!("{:?}", t2),
                                }
                            }
                        }
                        _ => todo!("{:?}", self.peek_token.typ),
                    }
                }
                TokenType::Punctuator(t) => match &t {
                    TokenPunctuator::LParen => {
                        v.push(self.parse_base_expression(Precedence::Lowest));
                    }
                    TokenPunctuator::Not => {
                        v.push(self.parse_base_expression(Precedence::Lowest));
                    }
                    _ => {
                        todo!("{:?}", &t)
                    }
                },
                _ => todo!("{:?}", self.current_token.typ),
            }
            self.next_token();
            self.skip_next_token_ptor(TokenPunctuator::Semicolon, true); //';'
        }
        v
    }
    //创建新的解析
    fn new_parser(&self, v: Vec<Token>) -> (Option<Expr>, Parser) {
        let list: Rc<RefCell<Vec<Token>>> = Rc::new(RefCell::new(v));
        let mut parser = Parser::new(Box::new(TokenList::new(Rc::clone(&list))));
        let expr = parser.parse_expression(0);
        if expr.len() != 1 {
            println!("{}", expr.len());
            for i in &expr {
                println!("{:?}", i);
            }
            panic!()
        }
        return (
            expr.first().unwrap().clone(),
            parser
        );
    }
    fn recursion_parse(&mut self) -> Option<Expr> {
        let mut list: Vec<Token> = Vec::new();
        self.next_token(); //(
        let mut paren = 1;
        let mut tk = self.current_token.clone();
        loop {
            if tk.is_ptor(TokenPunctuator::LParen) {
                paren += 1;
            } else if tk.is_ptor(TokenPunctuator::RParen) {
                paren -= 1;
            } else if tk.is_ptor(TokenPunctuator::Comma) {
                // if list.len() == 0 {
                //     panic!()
                // }
                // let (expr, _) = self.new_parser(list);
                // return expr;
                panic!()
            } else if tk.is_eof(true) {
                panic!("脚本异常")
            }
            if paren == 0{
                break;
            }
            list.push(tk);
            self.next_token();
            tk = self.current_token.clone();
        }
        let (expr, _) = self.new_parser(list);
        return expr;
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
    fn parse_base_expression(&mut self, infix: Precedence) -> Option<Expr> {
        let mut left: Expr = Expr::Empty;
        left = match &self.current_token.typ {
            TokenType::Ident(t) => {
                let ident = t.clone();
                if self.peek_token.is_ptor(TokenPunctuator::LParen) {
                    // a(
                    self.parse_call_slot(ident).unwrap()
                }
                else if self.peek_token.is_ptor(TokenPunctuator::LSParen){
                    //a[
                    self.parse_member_slot(ident).unwrap()
                }
                else {
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
                                Expr::Prefix(prefix, Box::new(self.parse_call_slot(ident)?))
                            }
                            else if self.peek_token.is_ptor(TokenPunctuator::LSParen){
                                //a[
                                Expr::Prefix(prefix, Box::new(self.parse_member_slot(ident)?))
                            }
                            else{
                                Expr::Prefix(prefix, Box::new(Expr::Identifier(ident.clone())))
                            }
                        }
                        _ => panic!(),
                    }
                } else {
                    match &t {
                        TokenPunctuator::Semicolon => Expr::Empty,
                        TokenPunctuator::LParen => self.recursion_parse().expect("脚本异常"),
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
        Some(left)
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
        Expr::Infix(Box::new(left), op, Box::new(right.unwrap()))
    }
    //a++,a--,++a,--a
    fn parse_update_slot(&mut self, ident: String, op: Operator, prefix: bool) -> Option<Expr> {
        let expr = Expr::Update(Box::new(Expr::Identifier(ident)), op, prefix);
        self.next_token(); //skip ident
        self.next_token(); //skip ++
        self.skip_next_token_ptor(TokenPunctuator::Semicolon, true); //skip ;
        Some(expr)
    }
    //a(
    fn parse_call_slot(&mut self, ident: String) -> Option<Expr> {
        self.next_token(); //ident
        if self.skip_next_token_ptor(TokenPunctuator::RParen, true) {
            return Some(Expr::Call(Box::new(Expr::Identifier(ident)), Vec::new()));
        }
        let mut list: Vec<Vec<Token>> = Vec::new();
        list.push(Vec::new());
        self.next_token(); //(
        let mut paren = 1;
        let mut tk = self.current_token.clone();
        loop {
            if tk.is_eof(true) {
                panic!("函数表达式生成异常(1)")
            } else if tk.is_ptor(TokenPunctuator::Comma) {
                list.push(Vec::new());
                continue;
            } else if tk.is_ptor(TokenPunctuator::LParen) {
                paren += 1;
            } else if tk.is_ptor(TokenPunctuator::RParen) {
                paren -= 1;
            }
            if paren == 0 {
                break;
            }
            list.last_mut()?.push(tk);
            self.next_token();
            tk = self.current_token.clone()
        }
        let mut args = Vec::new();
        for i in &list {
            let (expr, _) = self.new_parser(i.to_vec());
            args.push(expr?);
        }
        return Some(Expr::Call(Box::new(Expr::Identifier(ident)), args));
    }
    //a[  逻辑大至同 parse_call_slot
    fn parse_member_slot(&mut self, ident: String)->Option<Expr>{
        self.next_token(); //ident
        if self.skip_next_token_ptor(TokenPunctuator::RSParen, true) {
            panic!("{}", self.err("脚本异常"))
        }
        let mut list: Vec<Vec<Token>> = Vec::new();
        list.push(Vec::new());
        self.next_token(); //(
        let mut paren = 1;
        let mut tk = self.current_token.clone();
        loop {
            if tk.is_eof(true) {
                panic!("函数表达式生成异常(1)")
            } else if tk.is_ptor(TokenPunctuator::Comma) {
                list.push(Vec::new());
                continue;
            } else if tk.is_ptor(TokenPunctuator::LSParen) {
                paren += 1;
            } else if tk.is_ptor(TokenPunctuator::RSParen) {
                paren -= 1;
            }
            if paren == 0 {
                break;
            }
            list.last_mut()?.push(tk);
            self.next_token();
            tk = self.current_token.clone()
        }
        let mut args = Vec::new();
        for i in &list {
            let (expr, _) = self.new_parser(i.to_vec());
            args.push(expr?);
        }
        return Some(Expr::Member(Box::new(Expr::Identifier(ident)), args));
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
