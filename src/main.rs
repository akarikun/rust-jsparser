mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser};

fn main() {
    let input = r#"
        //let x = 1234;
        //let y = 11+(22*(33-44));  
        let x = 1;
        y++;
        //if(x==1){}
    "#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    program.eval();
}
