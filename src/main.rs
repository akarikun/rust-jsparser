mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser};

fn println(color: i32, label: &str, msg: String) {
    println!("\x1b[{}m{} \x1b[39m {}", color, label, msg);
}
fn main() {
    let input = r#"
        // let y = 11+(22*(33-44));  
        let x = 11+22*(33+44)-55;
        x++;
        a==b;
        c&&d;
        a==b&&c;
    "#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    program.eval();
}
