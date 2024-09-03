mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser};

fn main() {
    let input = r#"11"#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    //program.eval();
}
