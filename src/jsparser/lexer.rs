use super::token::{Token, TokenKeyword, TokenPunctuator, TokenType};

pub trait ILexer {
    fn next_token(&mut self) -> Token;
    fn new(input: String) -> Self;
}

pub struct Lexer {
    input: String,
    chars: std::vec::IntoIter<char>, // 拥有字符，Lexer 释放时一起释放
    position: usize,                 // 当前字符位置
    read_position: usize,            // 下一个字符的位置
    ch: Option<char>,                // 当前字符
    line: usize,                     // 当前行号
    column: usize,                   // 当前列号
}
impl ILexer for Lexer {
    fn new(input: String) -> Self {
        let mut lexer = Lexer {
            chars: input.chars().collect::<Vec<_>>().into_iter(),
            input,
            position: 0,
            read_position: 0,
            ch: None,
            line: 1,   // 初始行号为1
            column: 0, // 初始列号为0
        };
        lexer.read_char();
        lexer
    }

    fn next_token(&mut self) -> Token {
        if !self.skip_trivia() {
            return Token::new(TokenType::SyntaxError, self.line, self.column);
        }
        let token = match &self.ch {
            Some('=') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    //==
                    self.read_char();
                    let pc2 = self.peek_char();
                    if pc2 == Some('=') {
                        //===
                        self.read_char();
                        self.read_char();
                        return Token::new(
                            TokenType::Punctuator(TokenPunctuator::Congruent),
                            self.line,
                            self.column,
                        );
                    } else {
                        self.read_char();
                        return Token::new(
                            TokenType::Punctuator(TokenPunctuator::Equal),
                            self.line,
                            self.column,
                        );
                    }
                } else {
                    self.read_char();
                    return Token::new(
                        TokenType::Punctuator(TokenPunctuator::MOV),
                        self.line,
                        self.column,
                    );
                }
            }
            Some('+') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    //+=
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::ADD),
                        self.line,
                        self.column,
                    )
                } else if pc == Some('+') {
                    //++
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::INC),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Plus),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('-') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    //-=
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::SUB),
                        self.line,
                        self.column,
                    )
                } else if pc == Some('-') {
                    //--
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::DEC),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Minus),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('*') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::MUL),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Multiply),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('/') => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::DIV),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Divide),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('%') => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::MOD),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Modulo),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('(') => Token::new(
                TokenType::Punctuator(TokenPunctuator::LParen),
                self.line,
                self.column,
            ),
            Some(')') => Token::new(
                TokenType::Punctuator(TokenPunctuator::RParen),
                self.line,
                self.column,
            ),
            Some('{') => Token::new(
                TokenType::Punctuator(TokenPunctuator::LCParen),
                self.line,
                self.column,
            ),
            Some('}') => Token::new(
                TokenType::Punctuator(TokenPunctuator::RCParen),
                self.line,
                self.column,
            ),
            Some('[') => Token::new(
                TokenType::Punctuator(TokenPunctuator::LSParen),
                self.line,
                self.column,
            ),
            Some(']') => Token::new(
                TokenType::Punctuator(TokenPunctuator::RSParen),
                self.line,
                self.column,
            ),
            Some(';') => Token::new(
                TokenType::Punctuator(TokenPunctuator::Semicolon),
                self.line,
                self.column,
            ),
            Some(':') => Token::new(
                TokenType::Punctuator(TokenPunctuator::Colon),
                self.line,
                self.column,
            ),
            Some('.') => Token::new(
                TokenType::Punctuator(TokenPunctuator::Dot),
                self.line,
                self.column,
            ),
            Some(',') => Token::new(
                TokenType::Punctuator(TokenPunctuator::Comma),
                self.line,
                self.column,
            ),
            Some('^') => Token::new(
                TokenType::Punctuator(TokenPunctuator::BitXor),
                self.line,
                self.column,
            ),
            Some('~') => Token::new(
                TokenType::Punctuator(TokenPunctuator::BitNot),
                self.line,
                self.column,
            ),
            Some('`') | Some('"') | Some('\'') => {
                let p = self.ch.unwrap();
                let is_template = p == '`';
                self.read_char();
                let mut result = String::new();
                let mut v1: Vec<String> = Vec::new();
                let mut v2: Vec<String> = Vec::new();
                let line = self.line;
                let mut closed = false;
                while let Some(ch) = self.ch {
                    if !is_template && (ch == '\n' || ch == '\r') {
                        return Token::new(TokenType::SyntaxError, line, self.column);
                    }
                    if ch == '\\' {
                        let pc = self.peek_char();
                        match pc {
                            Some(t) => {
                                if t == p || t == '\\' {
                                    self.read_char();
                                    self.read_char();
                                    result.push(t);
                                    continue;
                                } else if t == '\n' {
                                    self.read_char();
                                    self.read_char();
                                    continue;
                                } else {
                                }
                            }
                            None => return Token::new(TokenType::SyntaxError, line, self.column),
                        }
                    }
                    if is_template {
                        if ch == '$' {
                            let pc = self.peek_char();
                            if pc.is_none() {
                                return Token::new(TokenType::SyntaxError, line, self.column);
                            }
                            if pc == Some('{') {
                                let mut count = 0;
                                self.read_char(); //$
                                v1.push(result.clone());
                                result.clear();
                                loop {
                                    if self.ch.is_none() {
                                        return Token::new(
                                            TokenType::SyntaxError,
                                            line,
                                            self.column,
                                        );
                                    }
                                    if self.ch == Some('{') {
                                        count += 1;
                                        self.read_char();
                                    } else if self.ch == Some('}') {
                                        count -= 1;
                                        self.read_char();
                                        if count == 0 {
                                            v2.push(result.clone());
                                            result.clear();
                                            break;
                                        }
                                    } else {
                                        result.push(self.ch.unwrap());
                                        self.read_char();
                                    }
                                }
                                continue;
                            }
                        }
                    }
                    if self.ch.is_none() {
                        break;
                    }
                    if self.ch.unwrap() == p {
                        self.read_char();
                        closed = true;
                        break;
                    }
                    result.push(self.ch.unwrap());
                    self.read_char();
                }
                if !closed {
                    return Token::new(TokenType::SyntaxError, line, self.column);
                }
                if is_template {
                    v1.push(result);
                    return Token::new(TokenType::Template(v1, v2), line, self.column);
                } else {
                    return Token::new(TokenType::StringLiteral(result), line, self.column);
                }
            }
            Some('&') => {
                let pc = self.peek_char();
                if pc == Some('&') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::And),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::BitAnd),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('|') => {
                let pc = self.peek_char();
                if pc == Some('|') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Or),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::BitOr),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('!') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::NE),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Not),
                        self.line,
                        self.column,
                    )
                }
            }
            Some(ch) if ch.is_digit(10) => {
                let (num, line) = self.read_number();
                return Token::new(TokenType::Literal(num.to_string()), line, self.column);
            }
            Some(ch) if ch.is_alphabetic() => {
                let (ident, line) = self.read_identifier();
                match ident.as_str() {
                    "let" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Let),
                            line,
                            self.column,
                        );
                    }
                    "const" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Const),
                            line,
                            self.column,
                        );
                    }
                    "var" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Var),
                            line,
                            self.column,
                        );
                    }
                    "if" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::If), line, self.column)
                    }
                    "else" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Else),
                            line,
                            self.column,
                        )
                    }
                    "return" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Return),
                            line,
                            self.column,
                        )
                    }
                    "break" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Break),
                            line,
                            self.column,
                        )
                    }
                    "continue" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Continue),
                            line,
                            self.column,
                        )
                    }
                    "for" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::For), line, self.column)
                    }
                    "in" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::In), line, self.column)
                    }
                    "of" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::Of), line, self.column)
                    }
                    "delete" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Delete),
                            line,
                            self.column,
                        )
                    }
                    "do" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::Do), line, self.column)
                    }
                    "switch" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Swith),
                            line,
                            self.column,
                        )
                    }
                    "case" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Case),
                            line,
                            self.column,
                        )
                    }
                    "default" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Default),
                            line,
                            self.column,
                        )
                    }
                    "function" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Function),
                            line,
                            self.column,
                        )
                    }
                    "while" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::While),
                            line,
                            self.column,
                        )
                    }
                    _ => {
                        return Token::new(TokenType::Ident(ident), line, self.column);
                    }
                }
            }
            Some('$') | Some('_') => {
                let (ident, line) = self.read_identifier();
                Token::new(TokenType::Ident(ident), self.line, self.column)
            }
            Some('>') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::GTE),
                        self.line,
                        self.column,
                    )
                } else if pc == Some('>') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::RShift),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::GT),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('<') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::LTE),
                        self.line,
                        self.column,
                    )
                } else if pc == Some('<') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::LShift),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::LT),
                        self.line,
                        self.column,
                    )
                }
            }
            None => Token::new(TokenType::EOF, self.line, self.column),
            _ => Token::new(TokenType::Illegal, self.line, self.column),
        };
        //这里会在最后再次读取/过滤下个字符,如果不需要该操作则需要提前return
        self.read_char();
        token
    }
}
impl Lexer {
    fn read_char(&mut self) -> bool {
        if let Some(ch) = self.chars.next() {
            self.ch = Some(ch);
            self.position = self.read_position;
            self.read_position += 1;

            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            true
        } else {
            self.ch = None;
            false
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.as_slice().first().copied()
    }

    // 注释是可重复的词法间隔；用循环跳过，不递归调用 next_token。
    fn skip_trivia(&mut self) -> bool {
        loop {
            while self.ch.is_some_and(char::is_whitespace) {
                self.read_char();
            }
            if self.ch != Some('/') {
                return true;
            }
            match self.peek_char() {
                Some('/') => {
                    while self.ch.is_some_and(|ch| ch != '\n') {
                        self.read_char();
                    }
                }
                Some('*') => {
                    self.read_char();
                    self.read_char();
                    loop {
                        if self.ch.is_none() {
                            return false;
                        }
                        if self.ch == Some('*') && self.peek_char() == Some('/') {
                            self.read_char();
                            self.read_char();
                            break;
                        }
                        self.read_char();
                    }
                }
                _ => return true,
            }
        }
    }

    fn read_number(&mut self) -> (String, usize) {
        let mut result = String::new();
        let line = self.line;
        while let Some(ch) = self.ch {
            if ch.is_digit(10) || ch == '.' {
                result.push(ch);
                self.read_char();
            } else {
                break;
            }
        }
        (result, line)
    }

    fn read_identifier(&mut self) -> (String, usize) {
        let mut result = String::new();
        let line = self.line;
        while let Some(ch) = self.ch {
            if ch == '$' || ch == '_' || ch.is_alphabetic() || ch.is_digit(10) {
                result.push(ch);
                self.read_char();
            } else {
                break;
            }
        }
        (result, line)
    }

    pub fn print(&mut self) {
        println!("/*--------print--------*/");
        let mut p = Lexer::new(self.input.clone());
        let mut line = 0;
        loop {
            let tok = p.next_token();
            if tok.typ == TokenType::EOF {
                break;
            }
            if line != tok.line && line > 0 {
                println!("");
            }
            line = tok.line;
            print!("{}", tok);
        }
        println!("\n/*-------- end --------*/");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn many_comments_do_not_recurse_and_compound_modulo_is_one_token() {
        let mut lexer = Lexer::new(format!("{}a%=2;", "/* comment */\n".repeat(10_000)));
        assert_eq!(lexer.next_token().typ, TokenType::Ident("a".into()));
        assert_eq!(
            lexer.next_token().typ,
            TokenType::Punctuator(TokenPunctuator::MOD)
        );
        assert_eq!(lexer.next_token().typ, TokenType::Literal("2".into()));
    }

    #[test]
    fn unterminated_strings_comments_and_templates_are_errors() {
        for source in ["'abc", "\"abc", "/* abc", "`abc", "`a${1", "`a${{x:1}"] {
            assert_eq!(
                Lexer::new(source.into()).next_token().typ,
                TokenType::SyntaxError,
                "{source}"
            );
        }
    }

    #[test]
    fn template_parts_include_the_empty_tail_and_plain_dollars() {
        let token = Lexer::new("`cost $5 ${1}`".into()).next_token();
        assert_eq!(
            token.typ,
            TokenType::Template(vec!["cost $5 ".into(), "".into()], vec!["1".into()])
        );
        assert_eq!(token.raw, "cost $5 ${1}");
    }
}
