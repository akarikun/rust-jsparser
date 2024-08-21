use super::{
    expr::{Expr, Infix, Prefix, Program, Stmt},
    lexer::Lexer,
    token::{Token, TokenType},
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
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        Program { statements }
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.current_token.typ {
            TokenType::Let => self.parse_let_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<Stmt> {
        self.next_token(); // skip 'let'

        let name = match &self.current_token.typ {
            TokenType::Ident(name) => name.clone(),
            _ => return None,
        };

        self.next_token(); // skip identifier

        if self.current_token.typ != TokenType::Assign {
            return None;
        }

        self.next_token(); // skip '='

        let expr = self.parse_expression(Precedence::Lowest)?;

        Some(Stmt::Let(name, expr))
    }

    fn parse_expression_statement(&mut self) -> Option<Stmt> {
        let expr = self.parse_expression(Precedence::Lowest)?;
        Some(Stmt::Expression(expr))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expr> {
        let mut left = match self.current_token.typ {
            TokenType::Ident(ref ident) => Some(Expr::Identifier(ident.clone())),
            TokenType::Number(ref num) => Some(Expr::Number(num.parse().unwrap())),
            TokenType::Minus => {
                self.next_token();
                let expr = self.parse_expression(Precedence::Prefix)?;
                Some(Expr::Prefix(Prefix::Negate, Box::new(expr)))
            }
            TokenType::LParen => {
                self.next_token();
                let expr = self.parse_expression(Precedence::Lowest)?;
                if self.current_token.typ != TokenType::RParen {
                    return None;
                }
                Some(expr)
            }
            _ => return None,
        }?;

        while precedence < self.peek_precedence() {
            left = match self.peek_token.typ {
                TokenType::Plus | TokenType::Minus | TokenType::Asterisk | TokenType::Slash => {
                    self.next_token();
                    left = self.parse_infix_expression(left);
                    left
                }
                TokenType::LParen => {
                    self.next_token();
                    left = self.parse_call_expression(left);
                    left
                }
                _ => return Some(left),
            };
        }

        Some(left)
    }

    fn parse_infix_expression(&mut self, left: Expr) -> Expr {
        let precedence = self.current_precedence();
        let infix_op = match self.current_token.typ {
            TokenType::Plus => Infix::Plus,
            TokenType::Minus => Infix::Minus,
            TokenType::Asterisk => Infix::Multiply,
            TokenType::Slash => Infix::Divide,
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
        if self.peek_token.typ == TokenType::RParen {
            self.next_token();
            return args;
        }

        self.next_token();

        args.push(self.parse_expression(Precedence::Lowest).unwrap());

        while self.peek_token.typ == TokenType::Comma {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Lowest).unwrap());
        }

        if self.peek_token.typ != TokenType::RParen {
            return vec![];
        }

        self.next_token(); // Skip ')'
        args
    }

    fn peek_precedence(&self) -> Precedence {
        match self.peek_token.typ {
            TokenType::Plus | TokenType::Minus => Precedence::Sum,
            TokenType::Asterisk | TokenType::Slash => Precedence::Product,
            TokenType::LParen => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    fn current_precedence(&self) -> Precedence {
        match self.current_token.typ {
            TokenType::Plus | TokenType::Minus => Precedence::Sum,
            TokenType::Asterisk | TokenType::Slash => Precedence::Product,
            TokenType::LParen => Precedence::Call,
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
