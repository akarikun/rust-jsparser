mod jsparser;
use jsparser::lexer::Lexer;

fn main() {
    let input = r#"
        let x = (15 + 10)*2;
        console.log(x);
    "#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();
}
