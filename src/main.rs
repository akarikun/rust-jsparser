mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser, program::JSType};
use std::time::Instant;

fn main() {
    //暂不支持连续赋值如: a=b=1;
    //最好每个语句最后结尾使用;结束
    let input = r#"
    log(a+1,2);
    log(a+1+a*2-a);
    log("a"+"1");
    log("a"+2+3);
    log(2+3);
    log(2+3+"a");
    if(a==1){ log("[1]:a==1"); } else{ log("[1]:a!=1"); }
    if(a==12){ log("[2]:a==12"); } else{ log("[2]:a!=1"); }
    log(foo2(1,2,3));
    function foo(a,b,c){return a+b+c;}
    function foo2(b,c){return a+b+c;}
    log(foo(1,2,3));
    log(foo2(2,3));
    log(foo3(1,2,3));//执行到这里报错后不执行后面的语句
    log(a+1);
"#;
    let start = Instant::now();
    let mut lexer = Lexer::new(String::from(input));
    // lexer.print();
    let mut parser = Parser::new(Box::new(lexer));
    let mut program = parser.parse_program();
    program.bind_global_value(String::from("a"), JSType::Int(12));
    program.register_method(
        String::from("log"),
        Box::new(|args| {
            println!("\x1b[33m log => {:?}\x1b[39m", args);
        }),
    );
    program.run();
    let duration = start.elapsed();
    let micros = duration.as_micros();
    let millis = duration.as_millis();
    println!("解析耗时: {:?}µs ({}ms)", micros, millis);
}
