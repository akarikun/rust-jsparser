use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::jsparser::{
    lexer::{ILexer, Lexer},
    parser::Parser,
    program::JSType,
};

pub fn err(str: &str) -> String {
    if cfg!(debug_assertions) {
        let msg = format!(
            //"\x1b[31m{}\x1b[39m,token:<\x1b[32m{}\x1b[39m>",
            "{}",
            str,
        );
        panic!("\x1b[31m{}\x1b[39m", msg);
    } else {
        let msg = format!(
            //"\x1b[31m{}\x1b[39m,token:<\x1b[32m{}\x1b[39m>",
            "{}",
            str,
        );
        return msg;
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    Err("webassembly暂不支持reqwest请求库".into())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true) // 忽略无效证书
        .build()?;
    let response = client.get(url).send()?;
    let result = response.text()?;
    Ok(result)
}

pub fn run_web(code: String, func: Box<dyn Fn(String) + Send + 'static>) -> Result<(), String> {
    let action = Arc::new(Mutex::new(func));

    let mut lexer = Lexer::new(code);
    lexer.print(); //打印token
    let mut parser = Parser::new(lexer);
    let program = Arc::new(Mutex::new(parser.parse_program()?));
    let pg = program.clone();

    if let Ok(mut pg_locked) = pg.try_lock() {
        pg_locked.print_tree(); //打印树

        //绑定全局变量
        pg_locked.bind_value(String::from("a"), JSType::Int(12));

        //注册全局方法
        pg_locked.register_method(
            String::from("log"),
            Box::new({
                let action = action.clone();
                move |args| {
                    // println!("\x1b[33m log => {:?}\x1b[39m", args);
                    // dbg!(&args);
                    action.lock().unwrap()(format!("\x1b[33m log => {:?}\x1b[39m", args));
                    return Ok(JSType::NULL);
                }
            }),
        );
        pg_locked.register_method(
            String::from("add"),
            Box::new({
                move |args| {
                    if args.len() > 0 {
                        let mut val = JSType::Int(0);
                        for i in args.clone() {
                            val = val.add(&i)?;
                        }
                        return Ok(val);
                    } else {
                        Ok(JSType::Int(0))
                    }
                }
            }),
        );

        pg_locked.register_method(
            "ajax".to_string(),
            Box::new({
                let action = action.clone();
                let pg = pg.clone();
                move |arg| {
                    if arg.len() == 0 {
                        action.lock().unwrap()(format!("ajax注册失败,缺少相关参数"));
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
                            //注:wasm不支持多线程,可正常编译但运行会出错
                            if cfg!(target_arch = "wasm32") {
                                match get(&url) {
                                    Ok(result) => {
                                        loop {
                                            if let Ok(mut pg) = pg.try_lock() {
                                                _ = pg.execute_func(
                                                    success.clone(),
                                                    vec![JSType::String(result)],
                                                );
                                                break;
                                            } else {
                                                std::thread::sleep(
                                                    std::time::Duration::from_millis(200),
                                                );
                                            }
                                        }
                                    }
                                    Err(err) => action.lock().unwrap()(format!("{:?}", err)),
                                }
                            } else {
                                std::thread::spawn({
                                    let pg = pg.clone();
                                    let success = success.clone();
                                    let action = action.clone();
                                    move || loop {
                                        std::thread::sleep(std::time::Duration::from_millis(200));
                                        match get(&url) {
                                            Ok(result) => {
                                                // 当循环多次结果:如果没有输出,执行到一定次数后程序会崩溃 memory allocation of 12288 bytes failed
                                                // action.lock().unwrap()(format!("{result}"));
                                                if let Ok(mut pg) = pg.try_lock() {
                                                    _ = pg.execute_func(
                                                        success.clone(),
                                                        vec![JSType::String(result)],
                                                    );
                                                    break;
                                                }
                                            }
                                            Err(err) => {
                                                action.lock().unwrap()(format!("{:?}", err));
                                                break;
                                            }
                                        };
                                    }
                                });
                            }
                        }
                    }
                    action.lock().unwrap()(format!("ajax注册成功"));
                    return Ok(JSType::NULL);
                }
            }),
        );
        pg_locked.run();
    } else {
        println!("程序异常");
    }

    Ok(())
}

pub fn run_console(code: String) -> Result<(), String> {
    // let count = Arc::new(Mutex::new(0));
    let start = Instant::now();
    _ = run_web(
        code,
        Box::new(move |msg| {
            println!("msg:{}", msg);
            // let mut i = count.lock().unwrap();
            // *i += 1;
            // println!("count:{}", *i);
        }),
    );
    let duration = start.elapsed();
    let micros = duration.as_micros();
    let millis = duration.as_millis();
    println!("解析耗时: {:?}µs ({}ms)\n", micros, millis);
    Ok(())
}
