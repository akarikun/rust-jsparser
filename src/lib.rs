mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser, program::JSType};
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn run_code(code: &str) {
    run(code.to_string()).unwrap();
}


pub fn run(code: String) -> Result<(), String> {
    let mut lexer = Lexer::new(String::from(code));
    // lexer.print(); //打印token
    let mut parser = Parser::new(Box::new(lexer));

    let mut program = parser.parse_program()?;
    // program.print_tree(); //打印树

    //绑定全局变量
    program.bind_value(String::from("a"), JSType::Int(12));
    //注册全局方法
    program.register_method(
        String::from("log"),
        Box::new(|args| {
            //println!("\x1b[33m log => {:?}\x1b[39m", args);
            log(&format!("\x1b[33m log => {:?}\x1b[39m", args));
            return Ok(JSType::NULL);
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
    Ok(())
}
