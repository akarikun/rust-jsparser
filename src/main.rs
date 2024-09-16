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
    a[b,c][d];
    a[b][c,d];
    a.b.c.d;
    let a=1+2*3
    alert(1);
    if(a<100){
        for(let i=0;i<100;i++){}
    }
    else if(a==100){
        alert(2)
        for(;;){};
    }
    else{
         for(let a in arr){};
    }   
    for(let a of arr){}
   
    "#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(Box::new(lexer));
    let program = parser.parse_program();
    program.eval();
}
