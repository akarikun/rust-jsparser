mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser, program::JSType};
use std::time::Instant;
fn main() -> Result<(), String> {
    let input = r#"
    //log(add(add(1,2,3),add(4,5)));
    //log(test(11));
    // log(test(11));
    //function test(val){
        for(let i = 0;i<10;i++){
            if (i%2==0)
                log("test:"+i+" "+(i+val+a)) 
            else
                log("test:"+i+" "+(i-val-a));
        }
    // }
    // log("------");
    // log(val);//val is not defined 执行到这里报错后不执行后面的语句
    // test(22);
"#;
    let start = Instant::now();
    let mut lexer = Lexer::new(String::from(input));
    lexer.print(); //打印token
    let mut parser = Parser::new(Box::new(lexer));

    let mut program = parser.parse_program()?;
    program.print_tree(); //打印树

    //绑定全局变量
    // program.bind_value(String::from("a"), JSType::Int(12));
    //注册全局方法
    program.register_method(
        String::from("log"),
        Box::new(|args| {
            println!("\x1b[33m log => {:?}\x1b[39m", args);
            return Ok(JSType::Void);
        }),
    );
    program.register_method(
        String::from("add"),
        Box::new(|args| {
            if args.len() > 0 {
                let mut val = JSType::Int(0);
                for i in args.clone() {
                    val = val.add(&i)?;
                }
                return Ok(val);
            } else {
                Ok(JSType::Int(0))
            }
        }),
    );
    program.run();
    let duration = start.elapsed();
    let micros = duration.as_micros();
    let millis = duration.as_millis();
    println!("解析耗时: {:?}µs ({}ms)", micros, millis);
    Ok(())
}
