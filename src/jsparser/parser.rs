use crate::jsparser::expr::Expression;

use super::{
    expr::{Expr, Infix, Prefix, Program, Stmt},
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

    fn log(&self) { println!("{:?},{:?}", &self.current_token.typ, &self.peek_token.typ);}
    
    fn err(&self,str:&str)->String { format!("{},line:{},column:{}",str,self.current_token.line,self.current_token.column) } 

    fn next_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }

    pub fn parse_program(&mut self) -> Program {
        println!("/*-------- parse_program --------/*");
        let mut statements = Vec::new();

        while self.current_token.typ != TokenType::EOF {
            println!("--|parse_program typ:{:?}", self.current_token.typ);
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }
        println!("/*-------- end --------/*");
        Program { statements }
    }

    fn checked_next_token(&mut self, typ:TokenPunctuator,to_next:bool)->bool{
        match &self.peek_token.typ {
            TokenType::Punctuator(t)=>{
                if typ == *t {
                    if to_next {
                        self.next_token();
                    }
                    return true;
                }
                return false;
            }
            _=> return false,
        }
    }
    fn parse_statement(&mut self) -> Option<Stmt> {
        println!("----|parse_statement typ:{:?}", self.current_token.typ);
        match &self.current_token.typ {
            TokenType::EOF => None,
            TokenType::Keyword(t) => {
                match t {
                    TokenKeyword::Let => {
                        let token = self.next_token(); // skip 'let'
                        println!("{:?}", token);
                        let name = match &self.current_token.typ {
                            TokenType::Ident(name) => name.clone(),
                            _ => return None,
                        };

                        self.next_token(); // skip identifier

                        if self.current_token.typ != TokenType::Punctuator(TokenPunctuator::Assign){
                            return None;
                        }
                        self.next_token(); // skip '='
                        let expr = self.parse_expression(Precedence::Lowest)?;
                        self.checked_next_token(TokenPunctuator::Semicolon,true); // skip ';'
                        // println!("expr {:?}",expr);
                        Some(Stmt::Variable("let".to_string(), name, expr))
                    }
                    _ => todo!(),
                }
            }
            TokenType::Punctuator(t) => match &t {
                TokenPunctuator::RParen => panic!("{}",self.err("多余')'")),
                // TokenPunctuator::Assign => todo!(),
                // TokenPunctuator::Plus => todo!(),
                // TokenPunctuator::Minus => todo!(),
                // TokenPunctuator::Asterisk => todo!(),
                // TokenPunctuator::Slash => todo!(),
                // TokenPunctuator::LParen => todo!(),
                // TokenPunctuator::Semicolon => todo!(),
                // TokenPunctuator::Dot => todo!(),
                // TokenPunctuator::Comma => todo!(),
                _=>{
                    println!("{:?}",t);
                    todo!()
                }
            },
            TokenType::Number(_) => {
                let expr = self.parse_expression(Precedence::Lowest)?;
                Some(Stmt::Expression(expr))
            }
            TokenType::Ident(t) => {
                let ident = t.clone();
                let expr = self.parse_expression(Precedence::Lowest)?;
               
                match &self.peek_token.typ{
                    TokenType::Punctuator(t2) => {
                        match &t2 {
                            TokenPunctuator::INC => {//++
                                let expr = Expr::Expression(Box::new(Expr::Identifier(ident)),t2.clone(),Expression::Update);
                                self.next_token();//skip ++
                                self.checked_next_token(TokenPunctuator::Semicolon,true);//skip ;
                                return Some(Stmt::Expression(expr));
                            },
                            _=>todo!()//return Some(Stmt::Expression(expr));
                        }
                    },
                    // TokenType::Keyword(_) => todo!(),
                    _=>todo!()
                }
                Some(Stmt::Expression(expr))
            }
            _ => todo!(),
        }
    }
    
    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expr> {
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
                        let expr = self.parse_expression(Precedence::Lowest)?;
                        if self.peek_token.typ == TokenType::Punctuator(TokenPunctuator::RParen) {
                            self.next_token();
                        }else{
                            panic!("{}",self.err("缺少')'"));
                        }
                        return Some(expr);
                    },
                    // TokenPunctuator::RParen => todo!(),
                    _=>todo!(),
                }
            },
            // TokenType::Illegal => todo!(),
            // TokenType::EOF => todo!(),
            // TokenType::Keyword(_) => todo!(),
            _=>todo!(),
        };
        while precedence < self.cp_precedence(&self.peek_token.typ) {
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
                    TokenPunctuator::LParen => {
                        self.next_token();
                        left = self.parse_call_expression(left);
                        left
                    }
                    _ => todo!()
                },
                _ => todo!()
            }
        }
       Some(left)
    }

    fn parse_infix_expression(&mut self, left: Expr) -> Expr {
        let precedence = self.cp_precedence(&self.current_token.typ);
        let infix_op = match self.current_token.typ {
            TokenType::Punctuator(TokenPunctuator::Plus) => Infix::Plus,
            TokenType::Punctuator(TokenPunctuator::Minus) => Infix::Minus,
            TokenType::Punctuator(TokenPunctuator::Asterisk) => Infix::Multiply,
            TokenType::Punctuator(TokenPunctuator::Slash) => Infix::Divide,
            _ => unreachable!(),
        };

        self.next_token(); // Skip operator
        let right = self.parse_expression(precedence);
        Expr::Infix(Box::new(left), infix_op, Box::new(right.unwrap()))
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

    fn cp_precedence(&self, typ: &TokenType) -> Precedence {
        match typ {
            TokenType::Punctuator(TokenPunctuator::Plus | TokenPunctuator::Minus) => {
                Precedence::Sum
            }
            TokenType::Punctuator(TokenPunctuator::Asterisk | TokenPunctuator::Slash) => {
                Precedence::Product
            }
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
