mod jsparser;
use jsparser::{
    lexer::Lexer,
    parser::Parser,
    program::{JSType, Program},
};
use reqwest::blocking::Client;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use std::{cell::RefCell, os::unix::thread};

fn main() -> Result<(), String> {
    _ = run(r#"
    ajax({
        url:'https://cdn.jsdelivr.net/npm/canvas-nest.js@2.0.4/dist/canvas-nest.min.js',
        type:'get',
        success:function(e){
            log(e);
        }
    })
"#
    .into());
    Ok(())
}

pub fn get(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(url).send()?;
    let result = response.text()?;
    Ok(result)
}
fn run(code: String) -> Result<(), String> {
    let start = Instant::now();
    let mut lexer = Lexer::new(String::from(code));
    lexer.print(); //打印token
    let mut parser = Parser::new(Box::new(lexer));

    let program = Arc::new(Mutex::new(parser.parse_program()?));
    let pg = program.clone();

    let mut pg_locked = pg.lock().unwrap();
    pg_locked.print_tree(); //打印树

    //绑定全局变量
    pg_locked.bind_value(String::from("a"), JSType::Int(12));
    //注册全局方法

    pg_locked.register_method(
        String::from("log"),
        Box::new(|args| {
            println!("\x1b[33m log => {:?}\x1b[39m", args);
            return Ok(JSType::NULL);
        }),
    );
    pg_locked.register_method(
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

    pg_locked.register_method(
        "ajax".to_string(),
        Box::new({
            let pg = program.clone();
            move |arg| {
                if arg.len() == 0 {
                    return Ok(JSType::NULL);
                }
                if let JSType::Object(json) = arg.get(0).unwrap() {
                    let url = json
                        .get("url")
                        .expect("缺少相关参数:url")
                        .to_string()
                        .unwrap();
                    let typ = json
                        .get("type")
                        .or(Some(&JSType::String("get".to_owned())))
                        .unwrap()
                        .to_string()
                        .unwrap()
                        .to_lowercase();
                    let success = json.get("success").expect("缺少相关参数:success");
                    if typ == "get".to_string() {
                        let result = get(&url).unwrap();
                        println!("{:?}", result);
                        //这里运行会死锁==!
                        // pg.lock().unwrap().execute_func(success.clone(), vec![JSType::String(result)]);
                    }
                }
                return Ok(JSType::NULL);
            }
        }),
    );
    pg_locked.run();    

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
    fn test_log() {
        _ = run("log(1);".into());
    }

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

    #[test]
    fn test_json() {
        let _ = run(r#"
            let json = {[1+1]:2} //[a+1]:5 暂未实现
        "#
        .to_string());
    }
}
