mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser};

fn println(color: i32, label: &str, msg: String) {
    println!("\x1b[{}m{} \x1b[39m {}", color, label, msg);
}
fn main() {
    //暂不支持连续赋值如: a=b=1;
    //不打算支持表达式中有未定义行为如：let i=i++;
    let input = r#"
        let a = b
        if(a==1) a=c; else if(b==2){
            alert(1);
        } else{
            if(c==d){} 
        }
    "#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(Box::new(lexer));
    let program = parser.parse_program();
    program.eval();
}
