use super::{
    expr::{Expr, Infix, Prefix, Program, Stmt},
    lexer::Lexer,
    token::{Token, TokenKeyword, TokenType,TokenPunctuator},
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

    fn next_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = Vec::new();

        while self.current_token.typ != TokenType::EOF {
            println!("parse_program {:?}",self.current_token.typ);
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        Program { statements }
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        println!("parse_statement {:?}",self.current_token.typ);
        match &self.current_token.typ {
            TokenType::EOF => None,
            TokenType::Keyword(t)=> {
                match t {
                    TokenKeyword::Let => {
                        let token = self.next_token(); // skip 'let'
                        println!("{:?}",token);
                        let name = match &self.current_token.typ {
                            TokenType::Ident(name) => name.clone(),
                            _ => return None,
                        };

                        self.next_token(); // skip identifier

                        if self.current_token.typ != TokenType::Punctuator(TokenPunctuator::Assign) {
                            return None;
                        }
                        self.next_token(); // skip '='
                        let expr = self.parse_expression(Precedence::Lowest)?;
                        Some(Stmt::Variable("let".to_string(),name, expr))
                    },
                    _ =>todo!()
                }
            },
            TokenType::Number(_)=>{
                let expr = self.parse_expression(Precedence::Lowest)?;
                println!("TokenType::Number:{:?}",expr);
                Some(Stmt::Expression(expr))
            },
            _ => todo!(),
        }
    }

    fn next_token_checked(&mut self,pass:bool) -> bool{
        let typ = &self.peek_token.typ ; 
        let r = match typ {
            TokenType::EOF => {
                return true;
            }
            TokenType::Punctuator(p)=>{
                match p {
                    TokenPunctuator::Semicolon => {
                        true
                    },
                    _=> false
                }
            }
            _=>false
        };
        if r && pass{
            self.next_token();
        }
        r
    }
    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expr> {
        let ck = &self.current_token.typ;
        let pk = &self.peek_token.typ;
        println!("{:?},{:?}",ck,pk);

        match &self.current_token.typ {
            TokenType::Ident(ident) =>{
                return Some(Expr::Identifier(ident.clone()));
            },
            TokenType::Number(num) => {
                let expr = Expr::Number(num.parse().unwrap());
                self.next_token_checked(true);
                return Some(expr);
            },
            TokenType::Illegal => todo!(),
            TokenType::EOF => todo!(),
            TokenType::Punctuator(_) => todo!(),
            TokenType::Keyword(_) => todo!(),
        }

        // let mut left = match &self.current_token.typ {
        //     //TokenType::Ident(ref ident) => Some(Expr::Identifier(ident.clone())),
        //     //TokenType::Number(ref num) => Some(Expr::Number(num.parse().unwrap())),
        //     TokenType::Punctuator(t) => {
        //         // if *t == TokenPunctuator::Minus {
        //         //     self.next_token();
        //         //     let expr = self.parse_expression(Precedence::Prefix)?;
        //         //     return Some(Expr::Prefix(Prefix::Negate, Box::new(expr)));
        //         // } else if *t == TokenPunctuator::LParen {
        //         //         self.next_token();
        //         //         let expr = self.parse_expression(Precedence::Lowest)?;
        //         //         if self.current_token.typ !=  TokenType::Punctuator(TokenPunctuator::RParen) {
        //         //             return None;
        //         //         }
        //         //         return Some(expr);
        //         // } else if *t == TokenPunctuator::Assign {
        //         //     self.next_token();
        //         //     let expr = self.parse_expression(Precedence::Lowest)?;
        //         //     if self.current_token.typ !=  TokenType::Punctuator(TokenPunctuator::RParen) {
        //         //         return None;
        //         //     }
        //         //     return Some(expr);
        //         // }
        //         None
        //     },
        //     // TokenType::Keyword(t)=>{
        //     //     match t {
        //     //         TokenKeyword::Let => {
        //     //             let expr = self.parse_expression(Precedence::Lowest).unwrap();
        //     //             Some(Expr::Variable(Box::new(expr)))
        //     //         },
        //     //         _=>todo!("parse_expression = Keyword"),
        //     //     }
        //     // },
        //     _ => todo!("parse_expression"),
        // }?;

        // // while precedence < self.peek_precedence() {
        // //     left = match &self.peek_token.typ {
        // //         TokenType::Punctuator(t)=>{
        // //             match t {
        // //                 TokenPunctuator::Plus | TokenPunctuator::Minus |TokenPunctuator::Asterisk|TokenPunctuator::Slash=>{
        // //                     self.next_token();
        // //                     left = self.parse_infix_expression(left);
        // //                     left
        // //                 },
        // //                 TokenPunctuator::LParen=>{
        // //                     self.next_token();
        // //                     left = self.parse_call_expression(left);
        // //                     left
        // //                 },
        // //                 _=> return Some(left),
        // //             }
        // //         },
        // //         _ => return Some(left),
        // //     };
        // // }
        // Some(left)
    }

    // fn parse_infix_expression(&mut self, left: Expr) -> Expr {
    //     let precedence = self.current_precedence();
    //     let infix_op = match self.current_token.typ {
    //         TokenType::Punctuator(TokenPunctuator::Plus) => Infix::Plus,
    //         TokenType::Punctuator(TokenPunctuator::Minus) => Infix::Minus,
    //         TokenType::Punctuator(TokenPunctuator::Asterisk) => Infix::Multiply,
    //         TokenType::Punctuator(TokenPunctuator::Slash) => Infix::Divide,
    //         _ => unreachable!(),
    //     };

    //     self.next_token(); // Skip operator
    //     let right = self.parse_expression(precedence);
    //     Expr::Infix(Box::new(left), infix_op, Box::new(right.unwrap()))
    // }

    // fn parse_call_expression(&mut self, function: Expr) -> Expr {
    //     let args = self.parse_call_arguments();
    //     Expr::Call(Box::new(function), args)
    // }

    fn parse_call_arguments(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        if self.peek_token.typ == TokenType::Punctuator(TokenPunctuator::RParen) {
            self.next_token();
            return args;
        }

        self.next_token();

        args.push(self.parse_expression(Precedence::Lowest).unwrap());

        while self.peek_token.typ == TokenType::Punctuator(TokenPunctuator::Comma) {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Lowest).unwrap());
        }

        if self.peek_token.typ != TokenType::Punctuator(TokenPunctuator::RParen) {
            return vec![];
        }

        self.next_token(); // Skip ')'
        args
    }

    fn peek_precedence(&self) -> Precedence {
        match self.peek_token.typ {
            TokenType::Punctuator(TokenPunctuator::Plus) | TokenType::Punctuator(TokenPunctuator::Minus) => Precedence::Sum,
            TokenType::Punctuator(TokenPunctuator::Asterisk) | TokenType::Punctuator(TokenPunctuator::Slash) => Precedence::Product,
            TokenType::Punctuator(TokenPunctuator::LParen) => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    fn current_precedence(&self) -> Precedence {
        match self.current_token.typ {
            TokenType::Punctuator(TokenPunctuator::Plus) | TokenType::Punctuator(TokenPunctuator::Minus) => Precedence::Sum,
            TokenType::Punctuator(TokenPunctuator::Asterisk) | TokenType::Punctuator(TokenPunctuator::Slash) => Precedence::Product,
            TokenType::Punctuator(TokenPunctuator::LParen) => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }
}

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Sum,     // + -
    Product, // * /
    Prefix,  // -x
    Call,    // function call
}
