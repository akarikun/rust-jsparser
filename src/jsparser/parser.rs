use crate::jsparser::expr::Expression;

use super::{
    expr::{Expr, Operator, Prefix, Program, Stmt},
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
    fn parse_statement(&mut self) -> Option<Stmt> {
        match &self.current_token.typ {
            TokenType::Ident(t) => {
                let expr = self.parse_expression()?;
                Some(Stmt::Expression(expr))
            }
            TokenType::Keyword(t) => {
                self.next_token(); // skip 'let'
                let name = match &self.current_token.typ {
                    TokenType::Ident(name) => name.clone(),
                    _ => panic!("{}", self.err("脚本异常")),
                };
                //nam =
                if !self.skip_next_token_ptor(TokenPunctuator::MOV, true) {
                    panic!("{}", self.err("脚本异常"))
                }
                self.next_token();

                let expr = self
                    .parse_base_expression(&InfixPrecedence::Lowest, &LogicalPrecedence::Lowest)?;
                self.skip_next_token_ptor(TokenPunctuator::Semicolon, true); // skip ';'
                                                                             // println!("expr {:?}",expr);
                Some(Stmt::Variable("let".to_string(), name, expr))
            }
            TokenType::Punctuator(t)=>{
                if &TokenPunctuator::Semicolon == t{
                    return None;
                }
                todo!("{:?}", t)
            }
            _ => todo!("{:?}", self.current_token.typ),
        }
    }
    ///第二层 转换至Expr step: 2->2或2->3
    fn parse_expression(&mut self) -> Option<Expr> {
        match &self.current_token.typ {
            TokenType::Ident(t) => {
                let box_ident = Box::new(Expr::Identifier(t.clone()));
                match &self.peek_token.typ {
                    TokenType::Punctuator(t2) => {
                        match &t2 {
                            TokenPunctuator::INC => {
                                //++
                                let expr = Expr::Update(box_ident,Operator::INC, false);
                                self.next_token(); //skip ++
                                self.skip_next_token_ptor(TokenPunctuator::Semicolon, true); //skip ;
                                return Some(expr);
                            }
                            TokenPunctuator::Equal => {
                                // <a==>
                                self.next_token(); //skip ident
                                self.next_token(); //skip ==

                                //a==b      a==b;
                                if self.peek_token.is_eof_or_semicolon(){
                                    //a==b
                                    match &self.current_token.typ {
                                        TokenType::Ident(t) | TokenType::Number(t) => {
                                            let expr = Expr::Infix(
                                                box_ident,
                                                Operator::Equal,
                                                Box::new(Expr::Identifier(t.clone())),
                                            );
                                            return Some(expr);
                                        }
                                        _ => todo!(),
                                    }
                                }
                                //a==b+c;
                                if self.peek_token.is_operator(){
                                    let right = self.parse_base_expression(&InfixPrecedence::Lowest, &LogicalPrecedence::Lowest)?;
                                    self.next_token();
                                    let expr = Expr::Infix(
                                        box_ident,
                                        Operator::Equal,
                                        Box::new(right));
                                    return Some(expr);
                                }
                                //a== b&&c
                                if self.peek_token.is_logical(){//更改优先级顺序
                                    if let Some(left )= self.parse_base_expression(&InfixPrecedence::Lowest, &LogicalPrecedence::Lowest){
                                        match left {
                                            Expr::Infix(left,op,right)=>{
                                                let expr = Expr::Infix(
                                                    Box::new(Expr::Infix(box_ident,Operator::Equal,left)),
                                                    op,
                                                    right);
                                                return Some(expr);
                                            },
                                            _=>return None
                                        }
                                    }
                                }
                                
                                // self.log();
                                //a==b+c

                                return None;
                            }
                            TokenPunctuator::And | TokenPunctuator::Or => {
                                // && ||
                                let op = if t2 == &TokenPunctuator::And {
                                    Operator::And
                                } else {
                                    Operator::Or
                                };

                                self.next_token(); //skip ident
                                self.next_token(); //skip ==
                                let mut expr = self.parse_base_expression(
                                    &InfixPrecedence::Lowest,
                                    &LogicalPrecedence::Lowest,
                                )?;
                                self.skip_next_token_ptor(TokenPunctuator::Semicolon, true); //skip;
                                return Some(Expr::Infix(box_ident, op, Box::new(expr)));
                            }
                            _ => todo!("{:?}", t2),
                        }
                    }
                    _ => todo!("{:?}", self.peek_token.typ),
                }
            }
            _ => todo!("{:?}", self.current_token.typ),
        }
    }
    ///第三层(base) 转换至Expr step: 3->3
    fn parse_base_expression(
        &mut self,
        infix: &InfixPrecedence,
        logical: &LogicalPrecedence,
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
                    TokenPunctuator::Semicolon => Expr::Empty,
                    TokenPunctuator::LParen => {
                        self.next_token();
                        //遇到左符号 从头开始解析
                        let expr = self.parse_base_expression(
                            &InfixPrecedence::Lowest,
                            &LogicalPrecedence::Lowest,
                        )?;
                        if self.peek_token.typ == TokenType::Punctuator(TokenPunctuator::RParen) {
                            self.next_token();
                        } else {
                            panic!("{}", self.err("缺少')'"));
                        }
                        return Some(expr);
                    }
                    // TokenPunctuator::RParen => todo!(),
                    TokenPunctuator::Equal => {
                        // self.next_token();
                        // let expr1 = self.parse_base_expression(&infix, &LogicalPrecedence::And)?;

                        // println!("{:?}", expr1);
                        // self.log();
                        // // //a == b &&
                        // // let expr2 = self.parse_base_expression(&infix,&logical)?;
                        // // println!("{:?}",expr2);
                        // // self.log();
                        // return Some(expr1);
                        panic!()
                    }
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
                        left = self.parse_infix_expression(left);
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
    fn parse_logical_expression(&mut self, left: Expr) -> Expr {
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
