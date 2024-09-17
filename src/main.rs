mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser};

fn println(color: i32, label: &str, msg: String) {
    println!("\x1b[{}m{} \x1b[39m {}", color, label, msg);
}
fn main() {
    //暂不支持连续赋值如: a=b=1;
    //不打算支持表达式中有未定义行为如：let i=i++;
    //最好每个语句最后结尾使用;结束
    let input = r#"
    let bbb=~!~!~a()[1]()[1,2,3](11,22,33)[3](2)[1][1](2)[3].s,
    bb=1
    ss,
    a=b;
    s
    s2
    a++;
    if(a==b) a++; else { let c=d; } 
    if(a==b){}else a+b; let a=c;
    for(let a=0;a<10;a++) a++;
    for(let a of arr){}
    for(let a in arr){ a++; }
"#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(Box::new(lexer));
    let program = parser.parse_program();
    program.eval();
}
