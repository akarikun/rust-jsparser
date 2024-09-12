mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser};

fn println(color: i32, label: &str, msg: String) {
    println!("\x1b[{}m{} \x1b[39m {}", color, label, msg);
}
fn main() {//
    let input = r#"
        let a = !b[1*m-n]+c-d*e+-f*g(2*3-h) &&aa+2>=0||1<(bb*3-cc)&&abc-p(bbb-333+ccc);
    "#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(Box::new(lexer));
    let program = parser.parse_program();
    program.eval();
}