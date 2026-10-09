use std::collections::HashMap;

use super::{
    expr::{Expr, Operator, Unary, Variable},
    lexer::ILexer,
    program::Program,
    token::{Token, TokenKeyword, TokenPunctuator, TokenType},
};

pub struct Parser<T: ILexer> {
    lexer: T,
    current_token: Token,
    last_token_line: usize, // 最近消费 token 的行号，用于语句结束与后缀更新
    allow_return: bool,
    allow_break: bool,
    allow_continue: bool,
}

impl<T: ILexer> Parser<T> {
    pub fn new(mut lexer: T) -> Self {
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
            last_token_line: 0,
            allow_return: false,
            allow_break: false,
            allow_continue: false,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        while !self.current_token.is_eof(false) {
            statements.push(self.parse_statement()?);
        }
        Ok(Program::new(statements))
    }

    fn next_token(&mut self) -> Token {
        self.last_token_line = self.current_token.line;
        std::mem::replace(&mut self.current_token, self.lexer.next_token())
    }

    fn err(&self, message: &str) -> String {
        format!(
            "{} (line {}, column {}, token {:?})",
            message, self.current_token.line, self.current_token.column, self.current_token.typ
        )
    }

    fn consume(&mut self, punctuator: TokenPunctuator) -> bool {
        if self.current_token.is_ptor(punctuator) {
            self.next_token();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, punctuator: TokenPunctuator) -> Result<(), String> {
        if self.consume(punctuator.clone()) {
            Ok(())
        } else {
            Err(self.err(&format!("Expected '{}'", punctuator)))
        }
    }

    fn identifier(&mut self) -> Result<String, String> {
        match self.current_token.typ.clone() {
            TokenType::Ident(name) => {
                self.next_token();
                Ok(name)
            }
            _ => Err(self.err("Expected identifier")),
        }
    }

    // 语句负责关键字和结束符；表达式入口不会吃掉逗号、分号或右括号。
    fn parse_statement(&mut self) -> Result<Expr, String> {
        match self.current_token.typ.clone() {
            TokenType::Keyword(TokenKeyword::If) => self.parse_if(),
            TokenType::Keyword(TokenKeyword::For) => self.parse_for(),
            TokenType::Keyword(TokenKeyword::While) => self.parse_while(),
            TokenType::Keyword(TokenKeyword::Do) => self.parse_do_while(),
            TokenType::Keyword(TokenKeyword::Swith) => self.parse_switch(),
            TokenType::Keyword(TokenKeyword::Function) => self.parse_function(false),
            TokenType::Punctuator(TokenPunctuator::LCParen) => self.parse_block(),
            TokenType::Punctuator(TokenPunctuator::Semicolon) => {
                self.next_token();
                Ok(Expr::Empty)
            }
            TokenType::Keyword(TokenKeyword::Break | TokenKeyword::Continue) => {
                let is_break = self.current_token.is_keyword(TokenKeyword::Break);
                if (is_break && !self.allow_break) || (!is_break && !self.allow_continue) {
                    return Err(self.err("Illegal break/continue statement"));
                }
                let line = self.next_token().line;
                self.finish_statement(line)?;
                Ok(if is_break {
                    Expr::Break
                } else {
                    Expr::Continue
                })
            }
            TokenType::Keyword(TokenKeyword::Return) => {
                if !self.allow_return {
                    return Err(self.err("Illegal return statement"));
                }
                let line = self.next_token().line;
                let value = if self.current_token.line > line
                    || self.current_token.is_eof(true)
                    || self.current_token.is_ptor(TokenPunctuator::RCParen)
                {
                    Expr::Empty
                } else {
                    self.parse_sequence()?
                };
                self.finish_statement(self.last_token_line)?;
                Ok(Expr::Return(Box::new(value)))
            }
            _ => {
                let value = if matches!(
                    self.current_token.typ,
                    TokenType::Keyword(TokenKeyword::Let | TokenKeyword::Var | TokenKeyword::Const)
                ) {
                    self.parse_variable()?
                } else {
                    self.parse_sequence()?
                };
                // 只有语句层消费结束符，避免内层表达式越过调用参数或 for 的边界。
                self.finish_statement(self.last_token_line)?;
                Ok(value)
            }
        }
    }

    fn finish_statement(&mut self, line: usize) -> Result<(), String> {
        if self.consume(TokenPunctuator::Semicolon)
            || self.current_token.is_eof(false)
            || self.current_token.is_ptor(TokenPunctuator::RCParen)
            || self.current_token.line > line
        {
            Ok(())
        } else {
            Err(self.err("Expected ';' or end of statement"))
        }
    }

    fn parse_variable(&mut self) -> Result<Expr, String> {
        let kind = match self.next_token().typ {
            TokenType::Keyword(TokenKeyword::Let) => Variable::Let,
            TokenType::Keyword(TokenKeyword::Const) => Variable::Const,
            _ => Variable::Var,
        };
        let mut declarations = Vec::new();
        loop {
            let name = self.identifier()?;
            let value = if self.consume(TokenPunctuator::MOV) {
                self.parse_expression(0)?
            } else {
                if kind == Variable::Const {
                    return Err(self.err("Const requires an initializer"));
                }
                Expr::Empty
            };
            declarations.push((kind.clone(), name, value));
            if !self.consume(TokenPunctuator::Comma) {
                break;
            }
        }
        Ok(Expr::Variable(declarations))
    }

    fn parse_sequence(&mut self) -> Result<Expr, String> {
        let first = self.parse_expression(0)?;
        if !self.consume(TokenPunctuator::Comma) {
            return Ok(first);
        }
        let mut values = vec![first];
        loop {
            values.push(self.parse_expression(0)?);
            if !self.consume(TokenPunctuator::Comma) {
                break;
            }
        }
        Ok(Expr::Sequence(values))
    }

    // Precedence climbing：高优先级递归进右侧，同级的左结合运算留给外层循环。
    fn parse_expression(&mut self, min_precedence: u8) -> Result<Expr, String> {
        let mut left = self.parse_prefix()?;
        loop {
            let Some((precedence, right_associative, operator)) = self.infix() else {
                break;
            };
            if precedence < min_precedence {
                break;
            }
            let assignment = precedence == 1;
            if assignment && !matches!(left, Expr::Identifier(_)) {
                return Err(self.err("Invalid assignment target (only identifiers are supported)"));
            }
            self.next_token();
            let right = self.parse_expression(if right_associative {
                precedence
            } else {
                precedence + 1
            })?;
            left = match operator {
                Some(operator) => Expr::Infix(Box::new(left), operator, Box::new(right)),
                None => {
                    let Expr::Identifier(name) = left else {
                        unreachable!()
                    };
                    Expr::Assignment(vec![(name, right)])
                }
            };
        }
        Ok(left)
    }

    // 运算符、优先级、结合性只维护这一张表。
    fn infix(&self) -> Option<(u8, bool, Option<Operator>)> {
        use TokenPunctuator::*;
        let TokenType::Punctuator(token) = &self.current_token.typ else {
            return None;
        };
        let (precedence, operator) = match token {
            MOV => return Some((1, true, None)),
            ADD => (1, Operator::ADD),
            SUB => (1, Operator::SUB),
            MUL => (1, Operator::MUL),
            DIV => (1, Operator::DIV),
            MOD => (1, Operator::MOD),
            Or => (2, Operator::Or),
            And => (3, Operator::And),
            BitOr => (4, Operator::BitOr),
            BitXor => (5, Operator::BitXor),
            BitAnd => (6, Operator::BitAnd),
            Equal => (7, Operator::Equal),
            NE => (7, Operator::NE),
            GT => (8, Operator::GT),
            GTE => (8, Operator::GTE),
            LT => (8, Operator::LT),
            LTE => (8, Operator::LTE),
            LShift => (9, Operator::LShift),
            RShift => (9, Operator::RShift),
            Plus => (10, Operator::Plus),
            Minus => (10, Operator::Subtract),
            Multiply => (11, Operator::Multiply),
            Divide => (11, Operator::Divide),
            Modulo => (11, Operator::Modulo),
            _ => return None,
        };
        Some((precedence, precedence == 1, Some(operator)))
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        let value = match self.current_token.typ.clone() {
            TokenType::Ident(name) => {
                self.next_token();
                Expr::Identifier(name)
            }
            TokenType::Literal(value) => {
                self.next_token();
                Expr::Literal(value)
            }
            TokenType::StringLiteral(value) => {
                self.next_token();
                Expr::StringLiteral(value)
            }
            TokenType::Template(parts, substitutions) => {
                self.next_token();
                let mut values = Vec::new();
                for source in substitutions {
                    let mut parser = Self::new(T::new(source));
                    values.push(parser.parse_sequence()?);
                    if !parser.current_token.is_eof(false) {
                        return Err(parser.err("Unexpected token in template expression"));
                    }
                }
                Expr::Template(parts, values)
            }
            TokenType::Punctuator(TokenPunctuator::LParen) => {
                self.next_token();
                let value = self.parse_sequence()?;
                self.expect(TokenPunctuator::RParen)?;
                value
            }
            TokenType::Punctuator(TokenPunctuator::LSParen) => self.parse_array()?,
            TokenType::Punctuator(TokenPunctuator::LCParen) => self.parse_object()?,
            TokenType::Keyword(TokenKeyword::Function) => self.parse_function(true)?,
            TokenType::Punctuator(
                TokenPunctuator::Not
                | TokenPunctuator::BitNot
                | TokenPunctuator::Plus
                | TokenPunctuator::Minus,
            ) => {
                let unary = match self.next_token().typ {
                    TokenType::Punctuator(TokenPunctuator::Not) => Unary::Not,
                    TokenType::Punctuator(TokenPunctuator::BitNot) => Unary::BitNot,
                    TokenType::Punctuator(TokenPunctuator::Plus) => Unary::Plus,
                    _ => Unary::Minus,
                };
                return Ok(Expr::Unary(unary, Box::new(self.parse_expression(12)?)));
            }
            TokenType::Punctuator(TokenPunctuator::INC | TokenPunctuator::DEC) => {
                let operator = self.next_token().raw;
                let target = self.parse_expression(12)?;
                if !matches!(target, Expr::Identifier(_) | Expr::Member(_, _)) {
                    return Err(self.err("Invalid update target"));
                }
                return Ok(Expr::Update(Box::new(target), operator, true));
            }
            _ => return Err(self.err("Expected expression")),
        };
        self.parse_postfix(value)
    }

    // 调用和成员访问都是已解析表达式的后缀，支持任意链式组合。
    fn parse_postfix(&mut self, mut value: Expr) -> Result<Expr, String> {
        loop {
            match self.current_token.typ.clone() {
                TokenType::Punctuator(TokenPunctuator::LParen) => {
                    self.next_token();
                    let args = self.parse_list(TokenPunctuator::RParen)?;
                    value = Expr::Call(Box::new(value), args);
                }
                TokenType::Punctuator(TokenPunctuator::Dot) => {
                    self.next_token();
                    let property = match self.current_token.typ.clone() {
                        TokenType::Ident(name) => {
                            self.next_token();
                            name
                        }
                        TokenType::Keyword(_) => self.next_token().raw,
                        _ => return Err(self.err("Expected property name")),
                    };
                    value = Expr::Member(Box::new(value), Box::new(Expr::Identifier(property)));
                }
                TokenType::Punctuator(TokenPunctuator::LSParen) => {
                    self.next_token();
                    let property = self.parse_sequence()?;
                    self.expect(TokenPunctuator::RSParen)?;
                    value = Expr::Member(Box::new(value), Box::new(property));
                }
                TokenType::Punctuator(TokenPunctuator::INC | TokenPunctuator::DEC)
                    if self.current_token.line == self.last_token_line =>
                {
                    if !matches!(value, Expr::Identifier(_) | Expr::Member(_, _)) {
                        return Err(self.err("Invalid update target"));
                    }
                    let operator = self.next_token().raw;
                    return Ok(Expr::Update(Box::new(value), operator, false));
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_list(&mut self, end: TokenPunctuator) -> Result<Vec<Expr>, String> {
        let mut values = Vec::new();
        while !self.current_token.is_ptor(end.clone()) {
            values.push(self.parse_expression(0)?);
            if !self.consume(TokenPunctuator::Comma) {
                break;
            }
        }
        self.expect(end)?;
        Ok(values)
    }

    fn parse_array(&mut self) -> Result<Expr, String> {
        self.expect(TokenPunctuator::LSParen)?;
        Ok(Expr::Array(self.parse_list(TokenPunctuator::RSParen)?))
    }

    fn parse_object(&mut self) -> Result<Expr, String> {
        self.expect(TokenPunctuator::LCParen)?;
        let mut properties = HashMap::new();
        while !self.current_token.is_ptor(TokenPunctuator::RCParen) {
            let token = self.next_token();
            let (name, shorthand) = match token.typ {
                TokenType::Ident(name) => (name, true),
                TokenType::Literal(name) | TokenType::StringLiteral(name) => (name, false),
                TokenType::Keyword(_) => (token.raw, false),
                _ => return Err(self.err("Expected object property name")),
            };
            let value = if self.consume(TokenPunctuator::Colon) {
                self.parse_expression(0)?
            } else if shorthand {
                Expr::Ref(name.clone())
            } else {
                return Err(self.err("Expected ':' after property name"));
            };
            properties.insert(name, value);
            if !self.consume(TokenPunctuator::Comma) {
                break;
            }
        }
        self.expect(TokenPunctuator::RCParen)?;
        Ok(Expr::Object(properties))
    }

    fn parse_block(&mut self) -> Result<Expr, String> {
        self.expect(TokenPunctuator::LCParen)?;
        let mut statements = Vec::new();
        while !self.current_token.is_ptor(TokenPunctuator::RCParen) {
            if self.current_token.is_eof(false) {
                return Err(self.err("Unterminated block"));
            }
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenPunctuator::RCParen)?;
        Ok(Expr::Block(statements))
    }

    fn parse_function(&mut self, expression: bool) -> Result<Expr, String> {
        self.next_token(); // function
        let name = if self.current_token.is_ident() {
            Expr::Identifier(self.identifier()?)
        } else if expression {
            Expr::Empty
        } else {
            return Err(self.err("Expected function name"));
        };
        self.expect(TokenPunctuator::LParen)?;
        let mut args = Vec::new();
        while !self.current_token.is_ptor(TokenPunctuator::RParen) {
            args.push(Expr::Identifier(self.identifier()?));
            if !self.consume(TokenPunctuator::Comma) {
                break;
            }
        }
        self.expect(TokenPunctuator::RParen)?;
        let saved = (self.allow_return, self.allow_break, self.allow_continue);
        self.allow_return = true;
        self.allow_break = false;
        self.allow_continue = false;
        let body = self.parse_block();
        (self.allow_return, self.allow_break, self.allow_continue) = saved;
        Ok(Expr::Function(Box::new(name), args, Box::new(body?)))
    }

    fn parse_condition(&mut self) -> Result<Expr, String> {
        self.expect(TokenPunctuator::LParen)?;
        let test = self.parse_sequence()?;
        self.expect(TokenPunctuator::RParen)?;
        Ok(test)
    }

    fn parse_if(&mut self) -> Result<Expr, String> {
        self.next_token();
        let test = self.parse_condition()?;
        let consequent = self.parse_statement()?;
        let alternate = if self.current_token.is_keyword(TokenKeyword::Else) {
            self.next_token();
            self.parse_statement()?
        } else {
            Expr::Empty
        };
        Ok(Expr::If(
            Box::new(test),
            Box::new(consequent),
            Box::new(alternate),
        ))
    }

    fn parse_loop_body(&mut self) -> Result<Expr, String> {
        let saved = (self.allow_break, self.allow_continue);
        self.allow_break = true;
        self.allow_continue = true;
        let body = self.parse_statement();
        (self.allow_break, self.allow_continue) = saved;
        body
    }

    fn parse_while(&mut self) -> Result<Expr, String> {
        self.next_token();
        let test = self.parse_condition()?;
        let body = self.parse_loop_body()?;
        Ok(Expr::While(Box::new(test), Box::new(body)))
    }

    fn parse_do_while(&mut self) -> Result<Expr, String> {
        self.next_token();
        let body = self.parse_loop_body()?;
        if !self.current_token.is_keyword(TokenKeyword::While) {
            return Err(self.err("Expected while after do body"));
        }
        self.next_token();
        let test = self.parse_condition()?;
        self.consume(TokenPunctuator::Semicolon);
        Ok(Expr::DoWhile(Box::new(test), Box::new(body)))
    }

    fn parse_for(&mut self) -> Result<Expr, String> {
        self.next_token();
        self.expect(TokenPunctuator::LParen)?;
        let init = if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
            Expr::Empty
        } else if matches!(
            self.current_token.typ,
            TokenType::Keyword(TokenKeyword::Let | TokenKeyword::Var | TokenKeyword::Const)
        ) {
            self.parse_variable()?
        } else {
            self.parse_sequence()?
        };
        self.expect(TokenPunctuator::Semicolon)?;
        let test = if self.current_token.is_ptor(TokenPunctuator::Semicolon) {
            Expr::Empty
        } else {
            self.parse_sequence()?
        };
        self.expect(TokenPunctuator::Semicolon)?;
        let update = if self.current_token.is_ptor(TokenPunctuator::RParen) {
            Expr::Empty
        } else {
            self.parse_sequence()?
        };
        self.expect(TokenPunctuator::RParen)?;
        let body = self.parse_loop_body()?;
        Ok(Expr::For(
            Box::new(init),
            Box::new(test),
            Box::new(update),
            Box::new(body),
        ))
    }

    fn parse_switch(&mut self) -> Result<Expr, String> {
        self.next_token();
        let test = self.parse_condition()?;
        self.expect(TokenPunctuator::LCParen)?;
        let saved_break = self.allow_break;
        self.allow_break = true;
        let cases = self.parse_switch_cases();
        self.allow_break = saved_break;
        self.expect(TokenPunctuator::RCParen)?;
        Ok(Expr::Switch(Box::new(test), cases?))
    }

    fn parse_switch_cases(&mut self) -> Result<Vec<Expr>, String> {
        let mut cases = Vec::new();
        let mut has_default = false;
        while !self.current_token.is_ptor(TokenPunctuator::RCParen) {
            let test = if self.current_token.is_keyword(TokenKeyword::Case) {
                self.next_token();
                self.parse_sequence()?
            } else if self.current_token.is_keyword(TokenKeyword::Default) && !has_default {
                self.next_token();
                has_default = true;
                Expr::Empty
            } else {
                return Err(self.err("Expected case or a single default"));
            };
            self.expect(TokenPunctuator::Colon)?;
            let mut body = Vec::new();
            while !self.current_token.is_keyword(TokenKeyword::Case)
                && !self.current_token.is_keyword(TokenKeyword::Default)
                && !self.current_token.is_ptor(TokenPunctuator::RCParen)
            {
                body.push(self.parse_statement()?);
            }
            cases.push(Expr::SwitchCase(Box::new(test), body));
        }
        Ok(cases)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsparser::lexer::Lexer;

    fn expression(source: &str) -> Expr {
        let mut parser = Parser::new(Lexer::new(source.into()));
        let result = parser.parse_expression(0).unwrap();
        assert!(
            parser.current_token.is_eof(false),
            "Unconsumed token: {:?}",
            parser.current_token
        );
        result
    }

    fn literal(value: &str) -> Expr {
        Expr::Literal(value.into())
    }
    fn ident(value: &str) -> Expr {
        Expr::Identifier(value.into())
    }
    fn binary(left: Expr, operator: Operator, right: Expr) -> Expr {
        Expr::Infix(Box::new(left), operator, Box::new(right))
    }

    #[test]
    fn arithmetic_precedence_and_left_associativity() {
        assert_eq!(
            expression("1+2*3-4"),
            binary(
                binary(
                    literal("1"),
                    Operator::Plus,
                    binary(literal("2"), Operator::Multiply, literal("3"))
                ),
                Operator::Subtract,
                literal("4")
            )
        );
        assert_eq!(
            expression("8%3*2"),
            binary(
                binary(literal("8"), Operator::Modulo, literal("3")),
                Operator::Multiply,
                literal("2")
            )
        );
        assert_eq!(
            expression("10-3-2"),
            binary(
                binary(literal("10"), Operator::Subtract, literal("3")),
                Operator::Subtract,
                literal("2")
            )
        );
        assert_eq!(
            expression("(1+2)*3"),
            binary(
                binary(literal("1"), Operator::Plus, literal("2")),
                Operator::Multiply,
                literal("3")
            )
        );
    }

    #[test]
    fn logical_comparison_and_bitwise_precedence() {
        assert_eq!(
            expression("1<2 && 3<4"),
            binary(
                binary(literal("1"), Operator::LT, literal("2")),
                Operator::And,
                binary(literal("3"), Operator::LT, literal("4"))
            )
        );
        assert_eq!(
            expression("a | b == c"),
            binary(
                ident("a"),
                Operator::BitOr,
                binary(ident("b"), Operator::Equal, ident("c"))
            )
        );
        assert_eq!(
            expression("a || b && c"),
            binary(
                ident("a"),
                Operator::Or,
                binary(ident("b"), Operator::And, ident("c"))
            )
        );
    }

    #[test]
    fn unary_and_update_operands_do_not_swallow_binary_expressions() {
        assert_eq!(
            expression("-a*b"),
            binary(
                Expr::Unary(Unary::Minus, Box::new(ident("a"))),
                Operator::Multiply,
                ident("b")
            )
        );
        assert_eq!(
            expression("!-a"),
            Expr::Unary(
                Unary::Not,
                Box::new(Expr::Unary(Unary::Minus, Box::new(ident("a"))))
            )
        );
        assert_eq!(
            expression("++a+b--"),
            binary(
                Expr::Update(Box::new(ident("a")), "++".into(), true),
                Operator::Plus,
                Expr::Update(Box::new(ident("b")), "--".into(), false)
            )
        );
    }

    #[test]
    fn calls_members_and_containers_share_the_expression_entry() {
        let member = Expr::Member(
            Box::new(Expr::Call(Box::new(ident("f")), vec![literal("1")])),
            Box::new(ident("x")),
        );
        let indexed = Expr::Member(
            Box::new(member),
            Box::new(binary(literal("2"), Operator::Plus, literal("3"))),
        );
        assert_eq!(
            expression("f(1).x[2+3](4)"),
            Expr::Call(Box::new(indexed), vec![literal("4")])
        );
        let object = Expr::Object(HashMap::from([(
            "x".into(),
            Expr::Array(vec![literal("2"), literal("3")]),
        )]));
        assert_eq!(
            expression("f([1], {x:[2,3]},)"),
            Expr::Call(
                Box::new(ident("f")),
                vec![Expr::Array(vec![literal("1")]), object]
            )
        );
    }

    #[test]
    fn assignments_are_right_associative_and_literals_keep_their_type() {
        assert_eq!(
            expression("a=b=1"),
            Expr::Assignment(vec![(
                "a".into(),
                Expr::Assignment(vec![("b".into(), literal("1"))])
            )])
        );
        assert_eq!(
            expression("a+=b*=2"),
            binary(
                ident("a"),
                Operator::ADD,
                binary(ident("b"), Operator::MUL, literal("2"))
            )
        );
        assert_eq!(
            expression("f(123, '123')"),
            Expr::Call(
                Box::new(ident("f")),
                vec![literal("123"), Expr::StringLiteral("123".into())]
            )
        );
    }

    #[test]
    fn declarations_and_statement_boundaries() {
        let mut parser = Parser::new(Lexer::new("let a, b=2; a\nb;".into()));
        assert_eq!(
            parser.parse_statement().unwrap(),
            Expr::Variable(vec![
                (Variable::Let, "a".into(), Expr::Empty),
                (Variable::Let, "b".into(), literal("2"))
            ])
        );
        assert_eq!(parser.parse_statement().unwrap(), ident("a"));
        assert_eq!(parser.parse_statement().unwrap(), ident("b"));
        assert!(parser.current_token.is_eof(false));
    }

    #[test]
    fn nested_contexts_and_return_line_breaks() {
        let source =
            "function f(){while(a){while(b){break;} continue;} function g(){return;} return\n1;}";
        let mut parser = Parser::new(Lexer::new(source.into()));
        let Expr::Function(_, _, body) = parser.parse_statement().unwrap() else {
            panic!()
        };
        let Expr::Block(statements) = *body else {
            panic!()
        };
        assert_eq!(statements[2], Expr::Return(Box::new(Expr::Empty)));
        assert_eq!(statements[3], literal("1"));
        assert!(parser.current_token.is_eof(false));
    }

    #[test]
    fn invalid_inputs_return_errors_instead_of_panicking_or_stalling() {
        for source in [
            "1+;",
            "f(1;",
            "[1,",
            "({x:})",
            "1=2;",
            "(a+b)++;",
            "{",
            "a b;",
            "const a;",
            "for(;;i++;){}",
            "function f(){break;}",
            "while(a){function f(){continue;}}",
            "switch(a){default:;default:;}",
            "f().;",
            "f()[ ];",
            "switch(a){case 1:",
        ] {
            let mut parser = Parser::new(Lexer::new(source.into()));
            assert!(
                parser.parse_program().is_err(),
                "Accepted invalid source: {source}"
            );
        }
    }

    #[test]
    fn switch_cases_parse_without_interpreter_support() {
        let mut parser = Parser::new(Lexer::new(
            "switch(a){case 1:case 2:break;default:;}".into(),
        ));
        let Expr::Switch(_, cases) = parser.parse_statement().unwrap() else {
            panic!()
        };
        assert_eq!(cases.len(), 3);
        assert_eq!(
            cases[1],
            Expr::SwitchCase(Box::new(literal("2")), vec![Expr::Break])
        );
    }
}
