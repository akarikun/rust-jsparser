use std::{cell::RefCell, rc::Rc};

use crate::jsparser::expr::Expression;

use super::{
    expr::{self, Expr, Operator, Prefix, Program, Stmt},
    lexer::{ILexer, Lexer, TokenList},
    token::{Token, TokenKeyword, TokenPunctuator, TokenType},
};

pub struct Parser<'a> {
    // lexer: Lexer,
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
                TokenType::Ident(t) => {
                    let expr = self.parse_expression().unwrap();
                    v.push(Some(Stmt::Expression(expr)))
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
                    let expr = self
                        .parse_base_expression(&InfixPrecedence::Lowest, &LogicalPrecedence::Lowest)
                        .unwrap();
                    self.skip_next_token_ptor(TokenPunctuator::Semicolon, true); //';'
                    v.push(Some(Stmt::Variable(format!("{}", k), name, expr)));
                }
                TokenType::Punctuator(t) => match &t {
                    TokenPunctuator::Semicolon => v.push(None),
                    TokenPunctuator::LParen => {
                        v.push(Some(Stmt::Expression(self.parse_expression().unwrap())))
                    }
                    TokenPunctuator::Comma => {
                        self.next_token();
                        continue
                    },
                    _ => todo!("{:?}", t),
                },
                _ => todo!("{:?}", self.current_token.typ),
            }
        }
        v
    }

    ///第二层 转换至Expr step: 2->2或2->3
    fn parse_expression(&mut self) -> Option<Expr> {
        match &self.current_token.typ {
            TokenType::Ident(t) => {
                let ident = t.clone();
                match &self.peek_token.typ {
                    TokenType::EOF => Some(Expr::Identifier(ident)),
                    TokenType::Punctuator(t2) => match &t2 {
                        TokenPunctuator::INC => self.parse_update_slot(ident, Operator::INC, false),
                        TokenPunctuator::Equal => {
                            let expr = self.parse_logical_slot(ident);
                            println!("{:?}",expr);
                            expr
                        },
                        TokenPunctuator::And | TokenPunctuator::Or => {
                            self.parse_logical_slot_sub(ident, t2.clone())
                        }
                        TokenPunctuator::LParen => self.parse_call_slot(ident),
                        TokenPunctuator::Comma => {
                            self.next_token(); //ident
                            Some(Expr::Identifier(ident))
                        }
                        _ => todo!("{:?}", t2),
                    },
                    _ => todo!("{:?}", self.peek_token.typ),
                }
            }
            TokenType::Punctuator(t) => match &t {
                TokenPunctuator::LParen => Some(self.recursion_parse().expect("脚本异常")),
                _ => todo!("{:?}", &t),
            },
            _ => todo!("{:?}", self.current_token.typ),
        }
    }

    fn recursion_parse(&mut self) -> Option<Expr> {
        let v: Vec<Token> = Vec::new();
        let list = Rc::new(RefCell::new(v));
        self.next_token(); //(
        let mut paren = 1;
        let mut tk = self.current_token.clone();
        loop {
            if tk.is_ptor(TokenPunctuator::LParen) {
                paren += 1;
            } else if tk.is_ptor(TokenPunctuator::RParen) {
                paren -= 1;
                if paren == 0 {
                    // (<Expr>)
                    println!("{:?}", list.borrow().len());
                    let mut parser = Parser::new(Box::new(TokenList::new(Rc::clone(&list))));
                    if list.borrow().len() == 0 {
                        return None;
                    }
                    return parser.parse_expression();
                }
            } else if tk.is_ptor(TokenPunctuator::Comma) {
                if list.borrow().len() == 0 {
                    panic!()
                }
                self.log();
                let mut parser = Parser::new(Box::new(TokenList::new(Rc::clone(&list))));
                let expr = parser.parse_expression();
                if list.borrow().len() != 0 {
                    panic!()
                }
                self.log();

                self.log();
                return expr;
            } else if tk.is_eof(true) {
                panic!("脚本异常")
            }

            list.borrow_mut().push(tk);
            self.next_token();
            tk = self.current_token.clone();
        }
    }
    ///第三层(base) 转换至Expr step: 3->3
    fn parse_base_expression(
        &mut self,
        infix: &InfixPrecedence,
        logical: &LogicalPrecedence,
    ) -> Option<Expr> {
        let mut step = 0;
        let mut left = match &self.current_token.typ {
            TokenType::Ident(ident) => {
                step+=1;
                let ident = ident.clone();
                Expr::Identifier(ident)
            }
            TokenType::Number(num) => {
                step+=1;
                let expr = Expr::Number(num.parse().unwrap());
                expr
            }
            TokenType::Punctuator(t) => {
                match &t {
                    TokenPunctuator::Semicolon => Expr::Empty,
                    // TokenPunctuator::LParen => {
                    //     self.next_token();
                    //     //遇到左符号 从头开始解析
                    //     let expr = self.parse_base_expression(
                    //         &InfixPrecedence::Lowest,
                    //         &LogicalPrecedence::Lowest,
                    //     )?;
                    //     if self.peek_token.typ == TokenType::Punctuator(TokenPunctuator::RParen) {
                    //         self.next_token();
                    //     } else {
                    //         panic!("{}", self.err("缺少')'"));
                    //     }
                    //     return Some(expr);
                    // }
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
        //infix
        while infix < &self.get_infix_precedence(&self.peek_token.typ) {
            left = match &self.peek_token.typ {
                TokenType::Punctuator(t) => match t {
                    TokenPunctuator::Plus
                    | TokenPunctuator::Minus
                    | TokenPunctuator::Multiply
                    | TokenPunctuator::Divide => {
                        self.next_token();
                        left = self.parse_base_infix_expression(left);
                        left
                    }
                    _ => todo!(),
                },
                _ => todo!(),
            }
        }
        //logical
        while logical < &self.get_logical_precedence(&self.peek_token.typ) {
            left = match &self.peek_token.typ {
                TokenType::Punctuator(t) => match t {
                    TokenPunctuator::And | TokenPunctuator::Or | TokenPunctuator::Not => {
                        self.next_token();
                        left = self.parse_base_logical_expression(left);
                        left
                    }
                    _ => todo!(),
                },
                _ => todo!(),
            }
        }
        for _ in 0..step { self.next_token(); } 
        Some(left)
    }

    //a++,a--,++a,--a
    fn parse_update_slot(&mut self, ident: String, op: Operator, prefix: bool) -> Option<Expr> {
        let expr = Expr::Update(Box::new(Expr::Identifier(ident)), op, prefix);
        self.next_token(); //skip ident
        self.next_token(); //skip ++
        self.skip_next_token_ptor(TokenPunctuator::Semicolon, true); //skip ;
        Some(expr)
    }
    //a==
    fn parse_logical_slot(&mut self, ident: String) -> Option<Expr> {
        let box_ident = Box::new(Expr::Identifier(ident.clone()));
        // <a==>
        self.next_token(); //skip ident
        self.next_token(); //skip ==
                           //a==b      a==b;
        if self.peek_token.is_eof(true) {
            //a==b
            match &self.current_token.typ {
                TokenType::Ident(t) | TokenType::Number(t) => {
                    let expr = Expr::Infix(
                        box_ident,
                        Operator::Equal,
                        Box::new(Expr::Identifier(t.clone())),
                    );
                    self.next_token();
                    return Some(expr);
                }
                _ => todo!("{:?}", &self.current_token.typ),
            }
        }
        //a==b+c;
        if self.peek_token.is_operator() {
            let right =
                self.parse_base_expression(&InfixPrecedence::Lowest, &LogicalPrecedence::Lowest)?;
            self.next_token();
            let expr = Expr::Infix(box_ident, Operator::Equal, Box::new(right));
            return Some(expr);
        }
        //a== b&&c
        if self.peek_token.is_logical() {
            //更改优先级顺序
            if let Some(left) =
                self.parse_base_expression(&InfixPrecedence::Lowest, &LogicalPrecedence::Lowest)
            {
                match left {
                    Expr::Infix(left, op, right) => {
                        let expr = Expr::Infix(
                            Box::new(Expr::Infix(box_ident, Operator::Equal, left)),
                            op,
                            right,
                        );
                        return Some(expr);
                    }
                    _ => return None,
                }
            }
        }
        todo!("{:?}", &self.current_token.typ);
    }
    //a&&
    fn parse_logical_slot_sub(&mut self, ident: String, typ: TokenPunctuator) -> Option<Expr> {
        // && ||
        let op = if typ == TokenPunctuator::And {
            Operator::And
        } else {
            Operator::Or
        };
        self.next_token(); //skip ident
        self.next_token(); //skip ==
        let expr = self.parse_base_expression(&InfixPrecedence::Lowest, &LogicalPrecedence::Lowest)?;
        return Some(Expr::Infix(
            Box::new(Expr::Identifier(ident)),
            op,
            Box::new(expr),
        ));
    }

    //base返回Expr,否则返回Some(Expr)
    fn parse_base_infix_expression(&mut self, left: Expr) -> Expr {
        let precedence = self.get_infix_precedence(&self.current_token.typ);
        let op = match self.current_token.typ {
            TokenType::Punctuator(TokenPunctuator::Plus) => Operator::Plus,
            TokenType::Punctuator(TokenPunctuator::Minus) => Operator::Minus,
            TokenType::Punctuator(TokenPunctuator::Multiply) => Operator::Multiply,
            TokenType::Punctuator(TokenPunctuator::Divide) => Operator::Divide,
            _ => unreachable!(),
        };

        self.next_token(); // Skip operator
        let right = self.parse_base_expression(&precedence, &LogicalPrecedence::Lowest);
        Expr::Infix(Box::new(left), op, Box::new(right.unwrap()))
    }
    fn parse_base_logical_expression(&mut self, left: Expr) -> Expr {
        let precedence = self.get_logical_precedence(&self.current_token.typ);
        let op = match self.current_token.typ {
            TokenType::Punctuator(TokenPunctuator::And) => Operator::And,
            TokenType::Punctuator(TokenPunctuator::Or) => Operator::Or,
            TokenType::Punctuator(TokenPunctuator::Not) => panic!(),
            _ => unreachable!(),
        };

        self.next_token(); // Skip operator
        let right = self.parse_base_expression(&InfixPrecedence::Lowest, &precedence);
        Expr::Infix(Box::new(left), op, Box::new(right.unwrap()))
    }

    //a(
    fn parse_call_slot(&mut self, ident: String) -> Option<Expr> {
        //recursion_parse 同逻辑
        let mut args = Vec::new();
        self.next_token(); //ident
                           //无参数
        if self.skip_next_token_ptor(TokenPunctuator::RParen, true) {
            self.next_token(); //)
            return Some(Expr::Call(Box::new(Expr::Identifier(ident)), args));
        }
        let expr = self.recursion_parse();
        loop {
            if self.current_token.is_eof(true) {
                //1个参数
                args.push(expr?);
                return Some(Expr::Call(Box::new(Expr::Identifier(ident)), args));
            }
        }

        // if let Some(e) = expr {
        //     args.push(e);
        // }
        // println!("{:?}",args);
        // self.log();
        // if self.current_token.is_ptor(TokenPunctuator::Semicolon) {

        // }
        // if self.current_token.is_ptor(TokenPunctuator::Semicolon){
        //     return Some(Expr::Call(Box::new(Expr::Identifier(ident)), args));
        // }

        // return Some(Expr::Call(Box::new(Expr::Identifier(ident)), args));

        // self.next_token(); //(
        // let mut paren = 1;
        // let mut is_eof = false;
        // loop {
        //     let tk = self.next_token();
        //     if tk.is_ptor(TokenPunctuator::LParen) {
        //         paren += 1;
        //     } else if tk.is_ptor(TokenPunctuator::RParen) {
        //         paren -= 1;
        //         if paren == 0{ // (<Expr>)
        //             println!("{:?}",tk)
        //             // let mut parser = Parser::new(Box::new(TokenList::new(Rc::clone(&list))));
        //             // let expr = parser.parse_expression();
        //             // println!("{:?}",expr);
        //             // return expr;
        //         }
        //     }
        //     else if tk.is_eof(true) {
        //         panic!("");
        //     }
        //     if is_eof || paren == 0 {
        //         break;
        //     }
        // }
        None
        // self.log();
        // self.next_token(); //(
        // let mut paren = 1;
        // loop {
        //     if self.skip_next_token_ptor(TokenPunctuator::RParen, true) {
        //         paren += 1;
        //     } else if self.skip_next_token_ptor(TokenPunctuator::RParen, true) {
        //         paren -= 1;
        //         if paren == 0 {
        //             break;
        //         }
        //     }
        //     let expr = self.parse_expression().unwrap(); //递归调用第二步
        //     println!("{:?}", expr);
        //     // self.log();
        //     args.push(expr);
        //     if self.skip_next_token_ptor(TokenPunctuator::Comma, true) {
        //         panic!("")
        //     }
        // }
        // Some(Expr::Call(Box::new(Expr::Identifier(ident)), args))
        // // let args = self.parse_call_arguments();
        // // Some(Expr::Call(Box::new(Expr::Identifier(ident)), args))
    }

    fn parse_call_arguments(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        // self.log();
        if self.skip_next_token_ptor(TokenPunctuator::RParen, true) {
            return args;
        }
        self.next_token(); //(

        args.push(
            self.parse_base_expression(&InfixPrecedence::Lowest, &LogicalPrecedence::Lowest)
                .unwrap(),
        );

        while self.peek_token.typ == TokenType::Punctuator(TokenPunctuator::Comma) {
            self.next_token();
            self.next_token();
            args.push(
                self.parse_base_expression(&InfixPrecedence::Lowest, &LogicalPrecedence::Lowest)
                    .unwrap(),
            );
        }
        if self.peek_token.typ != TokenType::Punctuator(TokenPunctuator::RParen) {
            return vec![];
        }

        self.next_token(); // Skip ')'
        args
    }

    fn get_infix_precedence(&self, typ: &TokenType) -> InfixPrecedence {
        match typ {
            TokenType::Punctuator(TokenPunctuator::Plus | TokenPunctuator::Minus) => {
                InfixPrecedence::Sum
            }
            TokenType::Punctuator(TokenPunctuator::Multiply | TokenPunctuator::Divide) => {
                InfixPrecedence::Product
            }
            _ => InfixPrecedence::Lowest,
        }
    }
    fn get_logical_precedence(&self, typ: &TokenType) -> LogicalPrecedence {
        match typ {
            TokenType::Punctuator(TokenPunctuator::And) => LogicalPrecedence::And,
            TokenType::Punctuator(TokenPunctuator::Or) => LogicalPrecedence::Or,
            TokenType::Punctuator(TokenPunctuator::Not) => LogicalPrecedence::Not,
            _ => LogicalPrecedence::Lowest,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
enum InfixPrecedence {
    Lowest,
    Sum, // + -
    Product, // * /
         // Prefix,  // -x
}

#[derive(Debug, PartialEq, PartialOrd)]
enum LogicalPrecedence {
    Lowest,
    Or,  //||
    And, //&&
    Not, // !
}
