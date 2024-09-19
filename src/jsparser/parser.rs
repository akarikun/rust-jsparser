use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    expr::{Expr, Operator, Prefix, Unary, Variable},
    lexer::{ILexer, TokenList},
    program::Program,
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

    pub fn parse_program(&mut self) -> Program {
        let statements = self.filter_statement();
        Program {
            statements: statements,
            call_map: HashMap::new(),
            value_map:HashMap::new(),
        }
    }

    //这里还会过滤一次里层
    fn filter_statement(&mut self) -> Vec<Expr> {
        let set_seq = |seq: &mut Vec<Expr>, statements: &mut Vec<Expr>| {
            let len = seq.len();
            if len == 1 {
                statements.push(seq.last().unwrap().clone());
            } else if len > 0 {
                statements.push(Expr::Sequence(seq.clone()));
                *seq = Vec::new();
            }
        };
        let mut statements: Vec<Expr> = Vec::new();
        let mut seq: Vec<Expr> = Vec::new();
        while self.current_token.typ != TokenType::EOF {
            for expr in self.parse_statement(0, true) {
                if let Expr::Identifier(_) = expr {
                    seq.push(expr.clone());
                    continue;
                }
                if let Expr::Literal(_) = expr {
                    seq.push(expr.clone());
                    continue;
                }
                // if let Expr::TemplateLiteral(_,_) = expr {
                //     seq.push(expr.clone());
                //     continue;
                // }
                set_seq(&mut seq, &mut statements);
                match expr {
                    Expr::Empty | Expr::Identifier(_) | Expr::Literal(_) => {}
                    Expr::Unexpected(msg) => {
                        self.println(
                            31,
                            format!("Uncaught SyntaxError: Unexpected token {}", msg),
                        );
                    }
                    Expr::Return(_) | Expr::Break | Expr::Continue => {
                        self.println(31, format!("Illegal {:?} statement", expr));
                    }
                    _ => {
                        statements.push(expr.clone());
                    }
                }
            }
            set_seq(&mut seq, &mut statements);
        }
        statements
    }

    ///里层，从这里开始解析(核心)
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
                break;
            }
            if self.current_token.is_eof(false) {
                break;
            }
            if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                self.next_token();
                v.push(Expr::Empty);
            }
            if self.current_token.is_literal() {
                let expr = self.base_analyze();
                v.push(expr);
                if self.current_token.is_ptor(TokenPunctuator::Comma) {
                    self.next_token();
                    continue;
                }
                skip_semicolon(self);
            } else if self.current_token.is_ident() {
                if self.peek_token.is_ptor(TokenPunctuator::MOV) {
                    for expr in self.parse_mov_slot() {
                        v.push(expr);
                    }
                    skip_semicolon(self);
                } else {
                    let expr = self.base_analyze();
                    v.push(expr);
                    if self.current_token.is_ptor(TokenPunctuator::Comma) {
                        self.next_token();
                        continue;
                    }
                    skip_semicolon(self);
                }
            } else {
                match &self.current_token.typ {
                    TokenType::Keyword(t) => {
                        if matches!(
                            &t,
                            TokenKeyword::Let | TokenKeyword::Var | TokenKeyword::Const
                        ) {
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
                        } else if matches!(&t, TokenKeyword::If) {
                            v.push(self.parse_if_slot());
                            skip_semicolon(self);
                        } else if matches!(&t, TokenKeyword::For) {
                            v.push(self.parse_for_slot());
                            skip_semicolon(self);
                        } else if matches!(&t, TokenKeyword::Break) {
                            v.push(Expr::Break);
                            self.next_token();
                            skip_semicolon(self);
                        } else if matches!(&t, TokenKeyword::Continue) {
                            v.push(Expr::Continue);
                            self.next_token();
                            skip_semicolon(self);
                        } else if matches!(&t, TokenKeyword::Return) {
                            self.next_token();
                            if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
                                skip_semicolon(self);
                            }
                            v.push(Expr::Return(Box::new(Expr::Empty)));
                        } else {
                            panic!("{}", self.err("Unexpected token"));
                        }
                    }
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

    fn log(&self) {
        self.println(
            31,
            format!(
                "<{}>,\n     <{}>",
                &self.current_token.desc(),
                &self.peek_token.desc()
            ),
        );
    }

    fn println(&self, color: i32, msg: String) {
        println!("\x1b[{}m{} \x1b[39m ", color, msg);
    }

    fn err(&self, str: &str) -> String {
        format!(
            "\x1b[31m{}\x1b[39m,token:<\x1b[32m{}\x1b[39m>",
            str,
            self.current_token.desc()
        )
    }

    fn next_token(&mut self) -> Token {
        let token = self.current_token.clone();
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
        token
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
            let mov = self.current_token.clone(); //b
            if mov.is_eof(true) {
                //let a=b,d;
                v.push(Expr::Assignment(token.raw.clone(), Box::new(Expr::Empty)));
                break;
            }
            if mov.is_ptor(TokenPunctuator::Comma) {
                v.push(Expr::Assignment(token.raw.clone(), Box::new(Expr::Empty)));
                self.next_token();
                continue;
            }
            if token.line != self.current_token.line {
                if self.peek_token.is_ptor(TokenPunctuator::MOV)
                    || self.peek_token.is_ptor(TokenPunctuator::Comma)
                    || self.peek_token.is_ptor(TokenPunctuator::Semicolon)
                {
                    v.push(Expr::Assignment(token.raw.clone(), Box::new(Expr::Empty)));
                    continue;
                } else {
                    todo!()
                }
            }
            self.next_token();
            // let last_line = self.current_token.line;
            let expr = self.base_analyze();
            v.push(Expr::Assignment(token.raw.clone(), Box::new(expr)));
            if self.current_token.is_ptor(TokenPunctuator::Comma) {
                if self.peek_token.checked_keyword() {
                    panic!("{}", self.err("Unexpected token"))
                }
                self.next_token();
                if self.current_token.is_eof(true) {
                    panic!("{}", self.err("Unexpected token"))
                }
                if self.peek_token.is_ident_num_is_literal_template()
                    && self.current_token.line != self.peek_token.line
                {
                    v.push(Expr::Assignment(
                        self.current_token.raw.clone(),
                        Box::new(Expr::Empty),
                    ));
                    self.next_token();
                    break;
                }
            }
        }
        // dbg!(&v);
        v
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
            if !self.current_token.is_ptor(TokenPunctuator::RCParen) {
                panic!("{}", self.err("Unexpected end of input"));
            }
            self.next_token();
            return Expr::BlockStatement(expr);
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
            if !self.current_token.is_ptor(TokenPunctuator::RParen) {
                panic!("{}", self.err("Unexpected token"));
            }
            self.next_token(); //skip ')'
            let mut expr1 = Expr::Empty;
            let mut expr2 = Expr::Empty;
            let mut line = 0;
            if self.current_token.is_ptor(TokenPunctuator::LCParen) {
                self.next_token();
                let list: Vec<Token> =
                    self.get_token_duration(TokenPunctuator::LCParen, TokenPunctuator::RCParen);
                let (expr, _) = self.new_parser(list, 0, true);
                if !self.current_token.is_ptor(TokenPunctuator::RCParen) {
                    panic!("{}", self.err("Unexpected end of input"));
                }
                self.next_token();
                expr1 = Expr::BlockStatement(expr);
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
        if !self.current_token.is_ptor(TokenPunctuator::RParen) {
            panic!("{}", self.err("Unexpected end of input"))
        }
        self.next_token();
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
            if !self.current_token.is_ptor(TokenPunctuator::RSParen) {
                panic!("{}", self.err("Unexpected end of input"))
            }
            self.next_token(); //]
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
            } else if self.current_token.is_literal() {
                return (true, Expr::Literal(self.current_token.raw.clone()));
            }
            // else if self.current_token.is_template_literal() {
            //     return (true, Expr::TemplateLiteral(self.current_token.raw.clone()));
            // }
        }
        return (false, Expr::Empty);
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
                    Expr::Identifier(_) | Expr::Literal(_) /*| Expr::TemplateLiteral(_) */ => {
                        *left = Expr::Infix(Box::new(left.clone()), t, Box::new(expr.clone()));
                    }
                    Expr::Infix(_left, _op, _right) => {
                        if self.get_precedence_by_operator(&_op.clone())
                            < self.get_precedence(op.typ.clone())
                        {
                            *left = Expr::Infix(
                                Box::new(*_left.clone()),
                                _op.clone(),
                                Box::new(Expr::Infix(_right.clone(), t, Box::new(expr.clone()))),
                            );
                        } else {
                            *left = Expr::Infix(Box::new(left.clone()), t, Box::new(expr.clone()));
                        }
                    }
                    _ => {
                        panic!()
                    }
                }
            } else if self.current_token.is_ptor(TokenPunctuator::LParen) {
                let expr = self.base_analyze();
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
                    let expr = self.base_analyze();
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
    fn base_analyze(&mut self) -> Expr {
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
            } else if self.current_token.is_ptor(TokenPunctuator::Comma) {
                return expr;
            }
            return self.parser_infix(&mut expr, Precedence::Lowest);
        }
        if self.current_token.is_unary() {
            let unary = parse_unary(self);
            let expr = self.base_analyze();
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
            if self.current_token.line != self.peek_token.line {
                let token = self.next_token();
                if token.is_ident() {
                    return Expr::Identifier(token.raw.clone());
                } else if token.is_literal() {
                    return Expr::Literal(token.raw.clone());
                }
                /*else if token.is_template_literal() {
                    return Expr::TemplateLiteral(token.raw.clone());
                }*/
                else {
                    panic!()
                }
            }
            panic!()
        }
    }
}

trait IParse {
    fn get_operator(&self, token: &Token) -> Operator;
    fn get_precedence(&self, typ: TokenType) -> Precedence;
    fn get_precedence_by_operator(&self, typ: &Operator) -> Precedence;
}

impl IParse for Parser {
    fn get_operator(&self, token: &Token) -> Operator {
        let op = match &token.typ {
            TokenType::Punctuator(t) => match t {
                TokenPunctuator::Plus => Operator::Plus,
                TokenPunctuator::Minus => Operator::Subtract,
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
    fn get_precedence_by_operator(&self, typ: &Operator) -> Precedence {
        match typ {
            Operator::Plus | Operator::Subtract => Precedence::Sum,
            Operator::Multiply | Operator::Divide => Precedence::Product,
            _ => unreachable!(),
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
