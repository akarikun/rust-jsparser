use crate::jsparser::expr::Expression;

use super::{
    expr::{Expr, Infix, Logical, Prefix, Program, Stmt},
    lexer::Lexer,
    token::{Token, TokenKeyword, TokenPunctuator, TokenType},
};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
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
        println!(
            "cur:[{:?},{}],peek:[{:?},{}]",
            &self.current_token.typ,
            &self.current_token.index,
            &self.peek_token.typ,
            &self.peek_token.index,
        );
    }

    fn err(&self, str: &str) -> String {
        format!(
            "{},token:{:?},index:{},line:{},column:{}",
            str,
            self.current_token.typ,
            self.current_token.index,
            self.current_token.line,
            self.current_token.column
        )
    }

    fn next_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = Vec::new();
        while self.current_token.typ != TokenType::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }
        Program {
            statements: statements,
        }
    }

    fn checked_next_token(&mut self, typ: TokenPunctuator, is_skip: bool) -> bool {
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
    fn parse_statement(&mut self) -> Option<Stmt> {
        // println!("----|parse_statement typ:{:?}", self.current_token.typ);
        match &self.current_token.typ {
            TokenType::EOF => None,
            TokenType::Keyword(t) => {
                match t {
                    TokenKeyword::Let => {
                        self.next_token(); // skip 'let'
                        let name = match &self.current_token.typ {
                            TokenType::Ident(name) => name.clone(),
                            _ => panic!("{}", self.err("脚本异常")),
                        };
                        //nam =
                        if !self.checked_next_token(TokenPunctuator::Assign, true) {
                            panic!("{}", self.err("脚本异常"))
                        }
                        self.next_token();

                        let expr = self
                            .parse_expression(InfixPrecedence::Lowest, LogicalPrecedence::Lowest)?;
                        self.checked_next_token(TokenPunctuator::Semicolon, true); // skip ';'
                                                                                   // println!("expr {:?}",expr);
                        Some(Stmt::Variable("let".to_string(), name, expr))
                    }
                    TokenKeyword::If => {
                        self.next_token(); // skip 'if'
                        if !self.checked_next_token(TokenPunctuator::LParen, true) {
                            panic!("");
                        }
                        Some(Stmt::If())
                    }
                    _ => todo!("{:?}", self.current_token.typ),
                }
            }
            TokenType::Punctuator(t) => match &t {
                TokenPunctuator::RParen => panic!("{}", self.err("多余')'")),
                TokenPunctuator::LParen => {
                    self.next_token();
                    let stmt = self.parse_statement()?; //递归读取其他分支
                    if !self.checked_next_token(TokenPunctuator::RParen, true) {
                        panic!("{}", self.err("缺少')'"));
                    }
                    self.checked_next_token(TokenPunctuator::Semicolon, true);
                    Some(stmt)
                }
                TokenPunctuator::Semicolon => None,
                TokenPunctuator::Minus | TokenPunctuator::Not => {
                    let prefix = if t == &TokenPunctuator::Minus {
                        Prefix::Negate
                    } else {
                        Prefix::Not
                    };
                    self.next_token();
                    let expr =
                        self.parse_expression(InfixPrecedence::Lowest, LogicalPrecedence::Lowest)?;
                    Some(Stmt::Expression(Expr::Prefix(prefix, Box::new(expr))))
                }
                _ => todo!("{:?}", t),
            },
            TokenType::Number(_) => {
                let expr =
                    self.parse_expression(InfixPrecedence::Lowest, LogicalPrecedence::Lowest)?;
                Some(Stmt::Expression(expr))
            }
            TokenType::Ident(t) => {
                let ident = t.clone();
                let box_ident = Box::new(Expr::Identifier(ident));

                match &self.peek_token.typ {
                    TokenType::Punctuator(t2) => {
                        let p = t2.clone();
                        match &t2 {
                            TokenPunctuator::INC => {
                                //++
                                let expr =
                                    Expr::Expression(box_ident, t2.clone(), Expression::Update);
                                self.next_token(); //skip ++
                                self.checked_next_token(TokenPunctuator::Semicolon, true); //skip ;
                                return Some(Stmt::Expression(expr));
                            }
                            TokenPunctuator::Equal => {
                                // ==
                                self.next_token(); //skip ident
                                self.next_token(); //skip ==
                                let expr = self.parse_expression(
                                    InfixPrecedence::Lowest,
                                    LogicalPrecedence::Lowest,
                                )?;
                                let stmt =
                                    Stmt::Expression(Expr::Binary(box_ident, p, Box::new(expr)));
                                self.checked_next_token(TokenPunctuator::Semicolon, true); //skip;
                                self.log();
                                return Some(stmt);
                            }
                            TokenPunctuator::And | TokenPunctuator::Or => {
                                // && ||
                                let logical = if t2 == &TokenPunctuator::And {
                                    Logical::And
                                } else {
                                    Logical::Or
                                };

                                self.next_token(); //skip ident
                                self.next_token(); //skip ==                                
                                let mut expr = self.parse_expression(
                                    InfixPrecedence::Lowest,
                                    LogicalPrecedence::Lowest,
                                )?;
                                let stmt = Stmt::Expression(Expr::Logical(
                                    box_ident,
                                    logical,
                                    Box::new(expr),
                                ));
                                self.checked_next_token(TokenPunctuator::Semicolon, true); //skip;
                                return Some(stmt);
                            }
                            _ => todo!("{:?}", t2), //return Some(Stmt::Expression(expr));
                        }
                    }
                    // TokenType::Keyword(_) => todo!(),
                    _ => todo!("{:?}", &self.peek_token.typ),
                }
                // Some(Stmt::Expression(expr))
            }
            _ => todo!("{:?}", &self.current_token.typ),
        }
        //     self.next_token();
        // // let expr = self.parse_expression(InfixPrecedence::Lowest)?;
    }

    fn parse_expression(
        &mut self,
        infix: InfixPrecedence,
        logical: LogicalPrecedence,
    ) -> Option<Expr> {
        let mut left = match &self.current_token.typ {
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
                    TokenPunctuator::LParen => {
                        self.next_token();
                        let expr = self
                            .parse_expression(InfixPrecedence::Lowest, LogicalPrecedence::Lowest)?;
                        if self.peek_token.typ == TokenType::Punctuator(TokenPunctuator::RParen) {
                            self.next_token();
                        } else {
                            panic!("{}", self.err("缺少')'"));
                        }
                        return Some(expr);
                    }
                    // TokenPunctuator::RParen => todo!(),
                    TokenPunctuator::Equal => {
                        self.next_token();
                        let expr1 = self
                            .parse_expression(InfixPrecedence::Lowest, LogicalPrecedence::Lowest)?;

                        // println!("{:?}",expr1);
                        // self.log();
                        // let expr2 = self.parse_expression(InfixPrecedence::Lowest)?;
                        // println!("{:?}",expr2);
                        // self.log();
                        return Some(expr1);
                    }
                    _ => todo!("{:?}", t),
                }
            }
            _ => todo!(),
        };
        //infix
        while infix < self.get_infix_precedence(&self.peek_token.typ) {
            left = match &self.peek_token.typ {
                TokenType::Punctuator(t) => match t {
                    TokenPunctuator::Plus
                    | TokenPunctuator::Minus
                    | TokenPunctuator::Asterisk
                    | TokenPunctuator::Slash => {
                        self.next_token();
                        left = self.parse_infix_expression(left);
                        left
                    }
                    _ => todo!(),
                },
                _ => todo!(),
            }
        }
        while logical < self.get_logical_precedence(&self.peek_token.typ) {
            left = match &self.peek_token.typ {
                TokenType::Punctuator(t) => match t {
                    TokenPunctuator::And | TokenPunctuator::Or | TokenPunctuator::Not => {
                        self.next_token();
                        left = self.parse_logical_expression(left);
                        left
                    }
                    _ => todo!(),
                },
                _ => todo!(),
            }
        }
        Some(left)
    }

    fn parse_infix_expression(&mut self, left: Expr) -> Expr {
        let precedence = self.get_infix_precedence(&self.current_token.typ);
        let op = match self.current_token.typ {
            TokenType::Punctuator(TokenPunctuator::Plus) => Infix::Plus,
            TokenType::Punctuator(TokenPunctuator::Minus) => Infix::Minus,
            TokenType::Punctuator(TokenPunctuator::Asterisk) => Infix::Multiply,
            TokenType::Punctuator(TokenPunctuator::Slash) => Infix::Divide,
            _ => unreachable!(),
        };

        self.next_token(); // Skip operator
        let right = self.parse_expression(precedence, LogicalPrecedence::Lowest);
        Expr::Infix(Box::new(left), op, Box::new(right.unwrap()))
    }
    fn parse_logical_expression(&mut self, left: Expr) -> Expr {
        let precedence = self.get_logical_precedence(&self.current_token.typ);
        let op = match self.current_token.typ {
            TokenType::Punctuator(TokenPunctuator::And) => Logical::And,
            TokenType::Punctuator(TokenPunctuator::Or) => Logical::Or,
            TokenType::Punctuator(TokenPunctuator::Not) => Logical::Not,
            _ => unreachable!(),
        };

        self.next_token(); // Skip operator
        let right = self.parse_expression(InfixPrecedence::Lowest, precedence);
        Expr::Logical(Box::new(left), op, Box::new(right.unwrap()))
    }

    fn parse_call_expression(&mut self, function: Expr) -> Expr {
        let args = self.parse_call_arguments();
        Expr::Call(Box::new(function), args)
    }

    fn parse_call_arguments(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        if self.peek_token.typ == TokenType::Punctuator(TokenPunctuator::RParen) {
            self.next_token();
            return args;
        }

        self.next_token();

        args.push(
            self.parse_expression(InfixPrecedence::Lowest, LogicalPrecedence::Lowest)
                .unwrap(),
        );

        while self.peek_token.typ == TokenType::Punctuator(TokenPunctuator::Comma) {
            self.next_token();
            self.next_token();
            args.push(
                self.parse_expression(InfixPrecedence::Lowest, LogicalPrecedence::Lowest)
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
            TokenType::Punctuator(TokenPunctuator::Asterisk | TokenPunctuator::Slash) => {
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

#[derive(PartialEq, PartialOrd)]
enum InfixPrecedence {
    Lowest,
    Sum, // + -
    Product, // * /
         // Prefix,  // -x
}

#[derive(PartialEq, PartialOrd)]
enum LogicalPrecedence {
    Lowest,
    Or,  //||
    And, //&&
    Not, // !
}
