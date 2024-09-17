use std::{cell::RefCell, rc::Rc};

use super::{
    expr::{Expr, Operator, Prefix, Program, Unary, Variable},
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
            for expr in self.parse_statement(0, true) {
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

    /// a = 1;   a=1;
    fn parse_mov_slot(&mut self) -> Vec<Expr> {
        let mut v: Vec<Expr> = Vec::new();
        loop {
            if self.current_token.is_eof(true) {
               break;
            }
            if self.current_token.checked_keyword() {
                break;
            }
            let token = self.next_token(); //a
            let mov = self.next_token(); //b
            if mov.is_eof(true) {
                //let a=b,d;
                v.push(Expr::Assignment(token.raw.clone(), Box::new(Expr::Empty)));
                break;
            }
            if mov.is_ptor(TokenPunctuator::Comma) {
                v.push(Expr::Assignment(token.raw.clone(), Box::new(Expr::Empty)));
                continue;
            }
            // let last_line = self.current_token.line;
            let expr = self.parse_base_analyze();
            v.push(Expr::Assignment(token.raw.clone(), Box::new(expr)));
            if self.current_token.is_ptor(TokenPunctuator::Comma) {
                if self.peek_token.checked_keyword() {
                    panic!("{}", self.err("Unexpected token"))
                }
                self.next_token();
                if self.current_token.is_eof(true) {
                    panic!("{}", self.err("Unexpected token"))
                }
                if self.peek_token.is_ident_or_num() && self.current_token.line != self.peek_token.line{
                    v.push(Expr::Assignment(self.current_token.raw.clone(), Box::new(Expr::Empty)));
                    self.next_token();
                    break;
                }
            }
        }
        // dbg!(&v);
        v
    }
    ///最外层，从这里开始解析
    fn parse_statement(&mut self, count: usize, is_skip_semicolon: bool) -> Vec<Expr> {
        //是否需要跳过 ';' if for 只有一行时不需要处理
        let skip_semicolon = |p: &mut Self| {
            if p.current_token.is_ptor(TokenPunctuator::Semicolon) {
                if is_skip_semicolon {
                    p.next_token();
                }
                return true;
            }
            false
        };

        let mut v: Vec<Expr> = Vec::new();
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
            }
            if self.current_token.is_num() {
                let expr = self.parse_base_analyze();
                v.push(expr);
                skip_semicolon(self);
            } else if self.current_token.is_ident() {
                if self.peek_token.is_ptor(TokenPunctuator::MOV) {
                    for expr in self.parse_mov_slot() {
                        v.push(expr);
                    }
                    skip_semicolon(self);
                } else {
                    let expr = self.parse_base_analyze();
                    v.push(expr);
                    skip_semicolon(self);
                }
            } else {
                match &self.current_token.typ {
                    TokenType::Punctuator(t) => {}
                    TokenType::Keyword(t) => match &t {
                        TokenKeyword::None => todo!(),
                        TokenKeyword::Let | TokenKeyword::Var | TokenKeyword::Const => {
                            let token = self.next_token();
                            let key = if token.raw == "var" {
                                Variable::Var
                            } else if token.raw == "let" {
                                Variable::Let
                            } else {
                                Variable::Const
                            };
                            for i in self.parse_mov_slot() {
                                if let Expr::Assignment(left, right) = i {
                                    v.push(Expr::Variable(key.clone(), left, right));
                                }
                            }
                            skip_semicolon(self);
                        }
                        TokenKeyword::If => {
                            v.push(self.parse_if_slot());
                            skip_semicolon(self);
                        }
                        TokenKeyword::For => {
                            v.push(self.parse_for_slot());
                            skip_semicolon(self);
                        }
                        _ => {
                            todo!("{:?}", &t);
                        }
                    },
                    _ => {}
                }
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
        let mut parser = Self::new(Box::new(TokenList::new(Rc::clone(&list))));
        let is_skip_semicolon = if count == 0 { true } else { false };
        let expr = parser.parse_statement(count, is_skip_semicolon);
        if is_checked && !parser.current_token.is_eof(false) {
            panic!("{}", self.err("子解析异常"))
        }
        return (expr, parser);
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
            let line = self.current_token.line;
            let body = self.parse_statement(1, false);
            if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
            } else if line == self.current_token.line {
                panic!("{}", self.err("Use of future reserved word in strict mode"));
            }
            Expr::Expression(Box::new(body[0].clone()))
        }
    }

    fn parse_for_slot(&mut self) -> Expr {
        self.next_token(); //for

        let filter = |p: &mut Self| {
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
            let line = self.current_token.line;
            let body = self.parse_statement(1, false);
            if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
            } else if line == self.current_token.line {
                panic!("{}", self.err("Use of future reserved word in strict mode"));
            }
            Expr::Expression(Box::new(body[0].clone()))
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
                let expr = self.parse_statement(1, false);
                if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                    line = 0;
                    self.next_token();
                }
                expr1 = Expr::Expression(Box::new(expr[0].clone()));
            }
            if self.current_token.is_keyword(TokenKeyword::Else) {
                if line == self.current_token.line {
                    panic!("{}", self.err("Unexpected token"))
                }
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
    fn parse_call_slot(&mut self, callee: &mut Expr) -> Expr {
        let t = self.next_token(); //(
        if self.current_token.is_ptor(TokenPunctuator::RParen) {
            self.next_token(); //)
            return Expr::Call(Box::new(callee.clone()), Vec::new());
        }
        let list = self.get_token_duration(TokenPunctuator::LParen, TokenPunctuator::RParen);
        let (expr, mut parser) = self.new_parser(list, 1, false);
        let mut v = Vec::new();
        v.push(expr[0].clone());
        while parser.current_token.is_ptor(TokenPunctuator::Comma) {
            parser.next_token();
            let e = parser.parse_statement(1, true);
            v.push(e[0].clone());
        }
        if self.current_token.is_ptor(TokenPunctuator::RParen) {
            self.next_token();
        } else {
            panic!("{}", self.err("Unexpected end of input"))
        }
        Expr::Call(Box::new(callee.clone()), v)
    }
    //a[   a.
    fn parse_member_slot(&mut self, mem: &mut Expr) {
        let t = self.next_token(); // . or [
        if t.is_ptor(TokenPunctuator::Dot) {
            if self.current_token.is_ident() {
                let ident = self.next_token();
                if let Expr::Member(_, property) = mem {
                    *property = Box::new(Expr::Identifier(ident.raw));
                    return;
                }
            }
        } else if t.is_ptor(TokenPunctuator::LSParen) {
            let list = self.get_token_duration(TokenPunctuator::LSParen, TokenPunctuator::RSParen);
            let (expr, mut parser) = self.new_parser(list, 1, false);
            let mut v = Vec::new();
            v.push(expr[0].clone());
            while parser.current_token.is_ptor(TokenPunctuator::Comma) {
                parser.next_token();
                let e = parser.parse_statement(1, true);
                v.push(e[0].clone());
            }
            if v.len() == 1 {
                if let Expr::Member(_, property) = mem {
                    *property = Box::new(expr[0].clone());
                } else {
                    panic!()
                }
            } else {
                if let Expr::Member(_, property) = mem {
                    *property = Box::new(Expr::Sequence(v));
                } else {
                    panic!()
                }
            }
            if self.current_token.is_ptor(TokenPunctuator::RSParen) {
                self.next_token(); //]
            } else {
                panic!("{}", self.err("Unexpected end of input"))
            }
        } else {
            panic!()
        }
    }
    //这里还要处理多级 如: a()[1]  a[1]()    a[1]()[1]()...
    fn parse_call_or_member(&mut self) -> Expr {
        let token = self.next_token();
        let mut expr = Expr::Empty;
        loop {
            if self.current_token.is_eof(true) {
                break;
            }
            if self.current_token.is_ptor(TokenPunctuator::LParen) {
                if Expr::Empty == expr {
                    expr = Expr::Identifier(token.raw.clone());
                }
                expr = self.parse_call_slot(&mut expr);
            } else if self.current_token.is_ptor(TokenPunctuator::LSParen)
                || self.current_token.is_ptor(TokenPunctuator::Dot)
            {
                if Expr::Empty == expr {
                    expr = Expr::Member(
                        Box::new(Expr::Identifier(token.raw.clone())),
                        Box::new(Expr::Empty),
                    );
                } else {
                    expr = Expr::Member(Box::new(expr), Box::new(Expr::Empty));
                }
                self.parse_member_slot(&mut expr)
            } else {
                return expr;
            }
        }
        expr
    }
    fn get_base(&mut self) -> (bool, Expr) {
        if self.peek_token.is_ptor(TokenPunctuator::Comma)
            || self.peek_token.is_eof(true)
            || self.peek_token.checked_keyword()
            || self.peek_token.is_precedence()
            || self.peek_token.is_update()
        {
            if self.current_token.is_ident() {
                return (true, Expr::Identifier(self.current_token.raw.clone()));
            } else if self.current_token.is_num() {
                return (
                    true,
                    Expr::Number(self.current_token.raw.clone().parse().unwrap()),
                );
            }
        }
        return (false, Expr::Empty);
    }

    fn get_operator(&self, token: &Token) -> Operator {
        let op = match &token.typ {
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
        op
    }
    fn parser_infix(&mut self, left: &mut Expr, precedence: Precedence) -> Expr {
        if self.peek_token.is_eof(true) {
            return left.clone();
        }
        if self.peek_token.checked_keyword() {
            return left.clone();
        }
        if self.peek_token.is_ptor(TokenPunctuator::Comma) {
            return left.clone();
        }
        let mut last_pre_token = self.current_token.typ.clone();
        while precedence < self.get_precedence(self.current_token.typ.clone()) {
            let op = self.next_token();
            if self.current_token.is_eof(true) {
                self.next_token();
                break;
            }
            let (ok, ref expr) = self.get_base();
            if ok {
                self.next_token();
                let t = self.get_operator(&op);
                match &left {
                    Expr::Number(_) | Expr::Identifier(_) => {
                        *left = Expr::Infix(Box::new(left.clone()), t, Box::new(expr.clone()));
                    }
                    Expr::Infix(_left, _op, _right) => {
                        if self.get_precedence(last_pre_token) > self.get_precedence(op.typ.clone())
                        {
                            *left = Expr::Infix(Box::new(left.clone()), t, Box::new(expr.clone()));
                        } else {
                            *left = Expr::Infix(
                                Box::new(*_left.clone()),
                                _op.clone(),
                                Box::new(Expr::Infix(_right.clone(), t, Box::new(expr.clone()))),
                            );
                        }
                    }
                    _ => {
                        panic!()
                    }
                }
                last_pre_token = op.typ.clone();
            } else if self.current_token.is_ptor(TokenPunctuator::LParen) {
                let expr = self.parse_base_analyze();
                *left = Expr::Infix(
                    Box::new(left.clone()),
                    self.get_operator(&op),
                    Box::new(expr),
                )
            } else {
                if self.current_token.line != self.peek_token.line {
                    self.next_token();
                    return left.clone();
                } else if self.peek_token.is_ptor(TokenPunctuator::LParen)
                    || self.peek_token.is_ptor(TokenPunctuator::LSParen)
                    || self.peek_token.is_ptor(TokenPunctuator::Dot)
                {
                    let expr = self.parse_base_analyze();
                    *left = Expr::Infix(
                        Box::new(left.clone()),
                        self.get_operator(&op),
                        Box::new(expr),
                    )
                } else {
                    panic!("{}", self.err("Unexpected"))
                }
            }
        }
        // dbg!(&left);
        left.clone()
    }
    //解析基础语法，遇到 ; 或 关键字结束
    fn parse_base_analyze(&mut self) -> Expr {
        let parse_unary = |p: &mut Self| -> Unary {
            let token = p.next_token();
            match token.typ {
                TokenType::Punctuator(t) => match &t {
                    TokenPunctuator::Not => Unary::Not,
                    TokenPunctuator::Plus => Unary::Plus,
                    TokenPunctuator::Minus => Unary::Minus,
                    TokenPunctuator::BitNot => Unary::BitNot,
                    _ => panic!(),
                },
                _ => panic!(),
            }
        };
        if self.current_token.is_eof(true) {
            // return Expr::Empty;
            panic!()
        }
        if self.current_token.checked_keyword() {
            panic!()
        }
        let (ok, mut expr) = self.get_base();
        if ok {
            self.next_token();
            if self.current_token.is_update() {
                let op = self.next_token();
                return Expr::Update(Box::new(expr), op.raw.clone(), false);
            }
            return self.parser_infix(&mut expr, Precedence::Lowest);
        }
        if self.current_token.is_unary() {
            let unary = parse_unary(self);
            let expr = self.parse_base_analyze();
            return self.parser_infix(&mut Expr::Unary(unary, Box::new(expr)), Precedence::Lowest);
        }
        if self.current_token.is_ptor(TokenPunctuator::LParen) {
            self.next_token();
            let list: Vec<Token> =
                self.get_token_duration(TokenPunctuator::LParen, TokenPunctuator::RParen);
            let (expr, _) = self.new_parser(list, 1, true);
            if !self.current_token.is_ptor(TokenPunctuator::RParen) {
                panic!("{}", self.err("Unexpected end of input"));
            }
            self.next_token(); //)
            return expr[0].clone();
        } else {
            if self.peek_token.is_ptor(TokenPunctuator::LParen)
                || self.peek_token.is_ptor(TokenPunctuator::LSParen)
                || self.peek_token.is_ptor(TokenPunctuator::Dot)
            {
                let mut expr = self.parse_call_or_member();
                return self.parser_infix(&mut expr, Precedence::Lowest);
            }
            if self.peek_token.is_ptor(TokenPunctuator::INC) {}
            if self.current_token.line != self.peek_token.line {
                let token = self.next_token();
                if token.is_ident() {
                    return Expr::Identifier(token.raw.clone());
                } else if token.is_num() {
                    return Expr::Number(token.raw.clone().parse().unwrap());
                } else {
                    panic!()
                }
            }
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
