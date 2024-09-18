mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser};

fn main() {
    //暂不支持连续赋值如: a=b=1;
    //不打算支持表达式中有未定义行为如：let i=i++;
    //最好每个语句最后结尾使用;结束
    let input = r#"
   print(1*2*3-4/2);
"#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(Box::new(lexer));
    let mut program = parser.parse_program();
    program.register_method(
        String::from("print"),
        Box::new(|args| {
            println!("register_method:print=> {:?}", args);
        }),
    );
    program.eval(true);
}
