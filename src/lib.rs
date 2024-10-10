mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser, program::JSType};
use std::time::Instant;

fn run(code: String) -> Result<(), String> {
    let start = Instant::now();
    let mut lexer = Lexer::new(String::from(code));
    lexer.print(); //打印token
    let mut parser = Parser::new(Box::new(lexer));

    let mut program = parser.parse_program()?;
    program.print_tree(); //打印树

    //绑定全局变量
    program.bind_value(String::from("a"), JSType::Int(12));
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
    println!("解析耗时: {:?}µs ({}ms)\n", micros, millis);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fn1() {
        let _ = run(r#"
    function foo(a){
        for(let i=0;i<a;i++)
            log(i+" |");
        //log(i);                  //Uncaught ReferenceError: i is not defined
        return a;
    }
    //log(i);                      //Uncaught ReferenceError: i is not defined
"#
        .to_string());
    }
    #[test]
    fn test_fn2() {
        let _ = run(r#"
            let i = 0;
            function foo(){
                for(;;i++){
                    if(i<10)
                        log(i);
                    else
                        return;
                }
                log(123);
            }
            foo();
        "#
        .to_string());
    }
    #[test]
    fn test_for() {
                let _ = run(r#"
            for(let i = 0;i<10;i++){
                log(i);
            }
        "#
                .to_string());

                let _ = run(r#"
                let i = 0;
                for(;;){
                    if(i<10){
                        log(i);
                    }
                    else{
                        break;
                    }
                    i++;
                }
            "#
                    .to_string());

            let _ = run(r#"
            let i = 0;
            for(;;i++;){
                if(i<10)
                    log(i);
                else
                    break;
            }
        "#
            .to_string());
    }
   
}
