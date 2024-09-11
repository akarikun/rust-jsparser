mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser};

fn println(color: i32, label: &str, msg: String) {
    println!("\x1b[{}m{} \x1b[39m {}", color, label, msg);
}
fn main() {
    let input = r#"
        1+1+d+b                                         //Y
        1+1                                             //Y 在考虑是否禁止"非;换行"的逻辑
        +d+b                                            //Y
        a(1+1+d+b);                                     //Y
        let aa = a+(b-c) || (c&&d) &&e || (f*e);        //Y
        a+b&&c+d;                                       //Y
        let aabbb = a+b&&c+d;                           //Y
        let t=a+b+c;                                    //Y
        (a+b);                                          //Y
        a+(b*(c-d));                                    //Y
        a+(b*(c-d));                                    //Y
        let x = 11+22*(33+44)-55;                       //Y
        let y = a+(b*(c-d));                            //Y
        a();                                            //Y
        a(b());                                         //Y
        a(b(),c(),d(a+(b*(c-d))));                      //Y
        x++;                                            //Y
        a==b;                                           //Y
        c&&d;                                           //Y
        a==b&&c;                                        //Y
        c&&d;                                           //Y
        ;                                               //Y
        a;                                              //Y
        b                                               //Y
        c                                               //Y 
"#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(Box::new(lexer));
    let program = parser.parse_program();
    program.eval();
}