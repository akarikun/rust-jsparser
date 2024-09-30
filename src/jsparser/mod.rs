pub mod expr;
pub mod lexer;
pub mod parser;
pub mod program;
pub mod token;

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
