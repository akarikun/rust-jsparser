mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser, program::JSType};

fn main() {
    //暂不支持连续赋值如: a=b=1;
    //最好每个语句最后结尾使用;结束
    let input = r#"
    log(a+1);
    log(a+1+a*2-a);
    log("a"+"1");
    log("a"+2+3);
    log(2+3);
    log(2+3+"a");
    if(a==1){ log(1); } else{ log(2); }
    if(a==100){ log(3); } else{ log(4); }
    function foo(a,b,c){return a+b+c;}
    function foo2(b,c){return a+b+c;}
    log(foo(1,2,3));
    log(foo2(2,3));
"#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(Box::new(lexer));
    let mut program = parser.parse_program();
    program.bind_global_value(String::from("a"), JSType::Int(100));
    program.register_method(
        String::from("log"),
        Box::new(|args| {
            println!("register_method:log=> {:?}", args);
        }),
    );
    program.run();
}
