mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser};

fn main() {
    let input = r#"
        let x = 1 + 100;
    "#;
    let mut lexer = Lexer::new(String::from(input));
    // lexer.print();

    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if let Some(result) = program.eval() {
        println!("Result: {}", result);
    } else {
        println!("Evaluation error");
    }
}
