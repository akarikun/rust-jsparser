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
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let response = client.get(url).send()?;
    let result = response.text()?;
    Ok(result)
}

pub fn run_web(code: String, func: Box<dyn Fn(String) + Send + 'static>) -> Result<(), String> {
    run_with_fetch(code, func, |url| get(url).map_err(|err| format!("请求失败: {err:?}")))
}

// 请求完成后，由拥有 Program 的线程执行回调，避免持锁重入解释器。
fn run_with_fetch(
    code: String,
    func: Box<dyn Fn(String) + Send + 'static>,
    fetch: fn(&str) -> Result<String, String>,
) -> Result<(), String> {
    let action = Arc::new(Mutex::new(func));
    let requests = Arc::new(Mutex::new(Vec::<(String, JSType)>::new()));
    let mut lexer = Lexer::new(code);
    lexer.print();
    let mut program = Parser::new(lexer).parse_program()?;
    program.print_tree();
    program.bind_value("a".into(), JSType::Int(12));
    program.register_method(
        "log".into(),
        Box::new({
            let action = action.clone();
            move |args| {
                action.lock().unwrap()(format!("\x1b[33m log => {:?}\x1b[39m", args));
                Ok(JSType::NULL)
            }
        }),
    );
    program.register_method(
        "add".into(),
        Box::new(|args| {
            args.iter()
                .try_fold(JSType::Int(0), |val, arg| val.add(arg))
        }),
    );
    program.register_method(
        "ajax".into(),
        Box::new({
            let requests = requests.clone();
            move |args| {
                let Some(JSType::Object(json)) = args.first() else {
                    return Err("ajax需要一个对象参数".into());
                };
                let url = json.get("url").ok_or("缺少相关参数:url")?.to_string()?;
                let typ = match json.get("type") {
                    Some(value) => value.to_string()?.to_lowercase(),
                    None => "get".into(),
                };
                if typ != "get" {
                    return Err("ajax暂只支持GET请求".into());
                }
                let success = json.get("success").ok_or("缺少相关参数:success")?;
                if !matches!(success, JSType::Function(_, _, _)) {
                    return Err("ajax的success必须是函数".into());
                }
                requests
                    .lock()
                    .map_err(|_| "请求队列锁已损坏")?
                    .push((url, success.clone()));
                Ok(JSType::NULL)
            }
        }),
    );
    program.run();
    loop {
        // 先释放队列锁，再执行请求和回调；回调也可以提交新请求。
        let batch = std::mem::take(&mut *requests.lock().map_err(|_| "请求队列锁已损坏")?);
        if batch.is_empty() {
            break;
        }
        #[cfg(not(target_arch = "wasm32"))]
        let completed = {
            let jobs: Vec<_> = batch
                .into_iter()
                .map(|(url, success)| std::thread::spawn(move || (success, fetch(&url))))
                .collect();
            jobs.into_iter()
                .map(|job| job.join().map_err(|_| "请求线程异常退出".to_string()))
                .collect::<Result<Vec<_>, _>>()?
        };
        #[cfg(target_arch = "wasm32")]
        let completed: Vec<_> = batch
            .into_iter()
            .map(|(url, success)| (success, fetch(&url)))
            .collect();
        for (success, result) in completed {
            match result {
                Ok(body) => {
                    program.execute_func(success, vec![JSType::String(body)])?;
                }
                Err(err) => action.lock().unwrap()(err),
            }
        }
    }
    Ok(())
}

pub fn run_console(code: String) -> Result<(), String> {
    // let count = Arc::new(Mutex::new(0));
    let start = Instant::now();
    run_web(
        code,
        Box::new(move |msg| {
            println!("msg:{}", msg);
            // let mut i = count.lock().unwrap();
            // *i += 1;
            // println!("count:{}", *i);
        }),
    )?;
    let duration = start.elapsed();
    let micros = duration.as_micros();
    let millis = duration.as_millis();
    println!("执行总耗时（含请求与回调）: {:?}µs ({}ms)\n", micros, millis);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_fetch(url: &str) -> Result<String, String> {
        match url {
            "first" => Ok("first-body".into()),
            "second" => Ok("second-body".into()),
            _ => Err("mock request failed".into()),
        }
    }

    #[test]
    fn ajax_callbacks_can_enqueue_requests_and_release_captures() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let output = messages.clone();
        run_with_fetch(
            r#"
            ajax({url:'first', success:function(body, missing){
                log(body);
                ajax({url:'second', success:function(){log('done');}});
            }});
            log('script');
            "#
            .into(),
            Box::new(move |msg| output.lock().unwrap().push(msg)),
            mock_fetch,
        )
        .unwrap();
        let output = messages.lock().unwrap();
        assert_eq!(output.len(), 3);
        assert!(output[0].contains("script"));
        assert!(output[1].contains("first-body"));
        assert!(output[2].contains("done"));
        // Program、注册闭包与回调持有的输出引用都应被释放。
        assert_eq!(Arc::strong_count(&messages), 1);
    }

    #[test]
    fn string_numbers_stay_strings_and_modulo_uses_left_associativity() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let output = messages.clone();
        run_with_fetch(
            "log('123'+1); log(8%3*2);".into(),
            Box::new(move |msg| output.lock().unwrap().push(msg)),
            mock_fetch,
        )
        .unwrap();
        let output = messages.lock().unwrap();
        assert!(output[0].contains("String(\"1231\")"));
        assert!(output[1].contains("Int(4)"));
    }

    #[test]
    fn failed_request_reports_error_without_calling_success() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let output = messages.clone();
        run_with_fetch(
            "ajax({url:'bad',success:function(){log('unexpected');}});".into(),
            Box::new(move |msg| output.lock().unwrap().push(msg)),
            mock_fetch,
        )
        .unwrap();
        assert_eq!(*messages.lock().unwrap(), vec!["mock request failed"]);
        assert_eq!(Arc::strong_count(&messages), 1);
    }
}
