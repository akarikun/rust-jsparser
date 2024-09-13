mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser};

fn println(color: i32, label: &str, msg: String) {
    println!("\x1b[{}m{} \x1b[39m {}", color, label, msg);
}
fn main() {
    //不支持连续赋值如: a=b=1;
    //不支持表达式中有未定义行为如：let i=i++;
    let input = r#"
        let a = !b(1-m*n)/c&&a+2>=0||1<(-c)&&-p[-d-+c];
        if (a==b){
            let a=1,b,c=d;
        }
        else {
            alert(2);
        }
    "#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(Box::new(lexer));
    let program = parser.parse_program();
    program.eval();
}
