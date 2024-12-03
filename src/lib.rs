mod jsparser;
use jsparser::utility::run_web;
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn run_code(code: &str) {
    run_web(
        code.to_string(),
        Box::new(|msg| {
            //println!("{}", msg);
            log(&msg);
        }),
    )
    .unwrap();
}
