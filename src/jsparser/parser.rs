use std::{cell::RefCell, rc::Rc};

use super::{
    expr::{Expr, Operator, Program, Stmt},
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
    fn skip_next_token(&mut self, typ: TokenType, is_skip: bool) -> bool {
        if typ == self.peek_token.typ {
            if is_skip {
                self.next_token();
            }
            return true;
        }
        false
    }
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
                TokenType::Number(t) | TokenType::Ident(t) => {
                    for i in self.parse_expression(1) {
                        if let Some(expr) = i {
                            v.push(Some(Stmt::Expression(expr)));
                        } else {
                            panic!()
                        }
                    }
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
                        v.push(Some(Stmt::Unexpected(self.peek_token.desc())));
                    }
                    self.next_token(); //ident
                    let expr = self.parse_base_expression(Precedence::Lowest).unwrap();
                    self.skip_next_token_ptor(TokenPunctuator::Semicolon, true); //';'
                    v.push(Some(Stmt::Variable(format!("{}", k), name, expr)));
                }
                TokenType::Punctuator(t) => match &t {
                    TokenPunctuator::Semicolon => v.push(None),
                    TokenPunctuator::LParen => {
                        for i in self.parse_expression(1) {
                            if let Some(expr) = i {
                                v.push(Some(Stmt::Expression(expr)));
                            } else {
                                panic!()
                            }
                        }
                    }
                    TokenPunctuator::Comma => {
                        self.next_token();
                        continue;
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
                                        v.push(self.parse_call_slot(ident));
                                    }
                                    TokenPunctuator::Comma => {
                                        self.next_token(); //ident
                                        v.push(Some(Expr::Identifier(ident)))
                                    }
                                    TokenPunctuator::Semicolon => {
                                        self.next_token(); //ident
                                        v.push(Some(Expr::Identifier(ident)))
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
                        v.push(Some(self.recursion_parse().expect("脚本异常")));
                    }
                    _ => {
                        // if t.is_precedence(){
                        //     v.push(self.parse_base_expression(Precedence::Lowest));
                        // }else{
                        todo!("{:?}", &t)
                        // }
                    }
                },
                _ => todo!("{:?}", self.current_token.typ),
            }
            self.next_token();
        }
        v
    }
    //创建新的解析
    fn new_parser(&self, v: Vec<Token>) -> (Option<Expr>, Parser, Vec<Token>) {
        let list: Rc<RefCell<Vec<Token>>> = Rc::new(RefCell::new(v));
        let mut parser = Parser::new(Box::new(TokenList::new(Rc::clone(&list))));
        if list.borrow().len() == 0 {
            return (None, parser, Vec::new());
        }
        let expr = parser.parse_expression(0);
        if expr.len() != 1 {
            println!("{}", expr.len());
            for i in &expr {
                println!("{:?}", i);
            }
            panic!()
        }
        if list.borrow().len() > 0 {
            println!("{:?}", list.borrow());
            panic!()
        }
        return (
            expr.first().unwrap().clone(),
            parser,
            list.borrow().to_vec(),
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
                if paren == 0 {
                    let (expr, _, tks) = self.new_parser(list);
                    return expr;
                }
            } else if tk.is_ptor(TokenPunctuator::Comma) {
                if list.len() == 0 {
                    panic!()
                }
                let (expr, _, tks) = self.new_parser(list);
                return expr;
            } else if tk.is_eof(true) {
                panic!("脚本异常")
            }
            list.push(tk);
            self.next_token();
            tk = self.current_token.clone();
        }
    }
    ///第三层(base) 转换至Expr step: 3->3
    fn parse_base_expression(&mut self, infix: Precedence) -> Option<Expr> {
        let mut left: Expr = Expr::Empty;
        left = match &self.current_token.typ {
            TokenType::Ident(ident) => {
                let ident = ident.clone();
                Expr::Identifier(ident)
            }
            TokenType::Number(num) => {
                let expr = Expr::Number(num.parse().unwrap());
                expr
            }
            TokenType::Punctuator(t) => {
                match &t {
                    TokenPunctuator::Semicolon => Expr::Empty,
                    TokenPunctuator::LParen => {
                        self.next_token();
                        //遇到左符号 从头开始解析
                        let expr = self.parse_base_expression(Precedence::Lowest)?;
                        if self.peek_token.typ == TokenType::Punctuator(TokenPunctuator::RParen) {
                            self.next_token();
                        } else {
                            panic!("{}", self.err("缺少')'"));
                        }
                        return Some(expr);
                    }
                    // // TokenPunctuator::RParen => todo!(),
                    // TokenPunctuator::Equal => {
                    //     // self.next_token();
                    //     // let expr1 = self.parse_base_expression(&infix, &LogicalPrecedence::And)?;

                    //     // println!("{:?}", expr1);
                    //     // self.log();
                    //     // // //a == b &&
                    //     // // let expr2 = self.parse_base_expression(&infix,&logical)?;
                    //     // // println!("{:?}",expr2);
                    //     // // self.log();
                    //     // return Some(expr1);
                    //     panic!()
                    // }
                    _ => todo!("{:?}", t),
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
        //recursion_parse 同逻辑
        self.next_token(); //ident
                           //无参数
        if self.skip_next_token_ptor(TokenPunctuator::RParen, true) {
            self.next_token(); //)
            return Some(Expr::Call(Box::new(Expr::Identifier(ident)), Vec::new()));
        }
        self.next_token(); //(
        let mut Paren = 1;
        let mut arr: Vec<Vec<Token>> = Vec::new();
        arr.push(Vec::new());
        loop {
            let token = self.next_token().clone();
            if token.is_eof(true) {
                panic!("函数表达式生成异常(1)")
            } else if token.is_ptor(TokenPunctuator::Comma) {
                arr.push(Vec::new());
                continue;
            } else if token.is_ptor(TokenPunctuator::LParen) {
                Paren += 1;
            } else if token.is_ptor(TokenPunctuator::RParen) {
                Paren -= 1;
            }
            if Paren == 0 {
                break;
            }
            arr.last_mut()?.push(token);
        }

        let mut args = Vec::new();
        // for i in &arr {
        //     for j in i.clone(){
        //         print!("<{:?}>",j.raw);
        //     }
        //     println!("")
        // }
        for i in &arr {
            let (expr, _, tks) = self.new_parser(i.to_vec());
            args.push(expr?);
            if tks.len() > 0 {
                panic!("函数表达式生成异常(2)")
            }
        }
        return Some(Expr::Call(Box::new(Expr::Identifier(ident)), args));
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
    Prefix,     // !x (前缀运算符) -x, 暂时不处理(负号跟减号有冲突,非表达符号的逻辑与基本也不一致,非为!开始,基本逻辑表达式为字母数字开始,后期做优化)
}
