### Rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

| 表达式 | 标记 | 备注 | 解析前提条件 |
| --- | --- | --- | --- |
| a+b | \<base_expr> | 四则运算,bool表达式等，a(),a[1]这种都属于<br/>这里是最复杂的，还需要考虑a()\[1]()[1]...这种套娃的还有a.b这种复合类型等 | cur=ident <br /> peek= +-*/[(.等 |
| i = a+b 相当于 i=\<base_expr> | \<MOV> | 由于已经解析了base_expr，当cur,peek符合条件后<br />再从当前函数中取返回值 | cur=ident <br /> peek=mov |
| let i = a+b 相当于 let \<MOV> | \<var> | 同上，满足cur跟peek后还要再读取下个token符合'='后<br />再从当前函数中取返回值 | cur=< key:let ><br/>peek=ident |
```
mod jsparser;
use jsparser::{lexer::Lexer, parser::Parser, program::JSType};
use std::time::Instant;
fn main() -> Result<(), String> {
    let input = r#"
    function foo(a){
        let i=0;
        for(;i<a;i++){
            log(i+" |");
        }
        return a;
    }
    foo(10);
    log("--------");
    let t = 1+foo(20);
    log("--------");
    log(t);
"#;
    let start = Instant::now();
    let mut lexer = Lexer::new(String::from(input));
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
    println!("解析耗时: {:?}µs ({}ms)", micros, millis);
    Ok(())
}

```
```
/*--------print--------*/
<key:function> <foo> <(> <a> <)> <{>
<key:let> <i> <=> <0> <;>
<key:for> <(> <;> <i> <<> <a> <;> <i> <++> <)> <{>
<log> <(> <i> <+> <" |"> <)> <;>
<}>
<key:return> <a> <;>
<}>
<foo> <(> <10> <)> <;>
<log> <(> <"--------"> <)> <;>
<key:let> <t> <=> <1> <+> <foo> <(> <20> <)> <;>
<log> <(> <"--------"> <)> <;>
<log> <(> <t> <)> <;>
/*-------- end --------*/
/*--------tree--------*/
(1) | Function(Identifier("foo"), [Identifier("a")], BlockStatement([Variable2([(Let, "i", Literal("0", "0"))]), For(Empty, Infix(Identifier("i"), LT, Identifier("a")), Update(Identifier("i"), "++", false), BlockStatement([Call(Identifier("log"), [Infix(Identifier("i"), Plus, Literal(" |", "\" |\""))])])), Return(Identifier("a"))]))
(2) | Call(Identifier("foo"), [Literal("10", "10")])
(3) | Call(Identifier("log"), [Literal("--------", "\"--------\"")])
(4) | Variable2([(Let, "t", Infix(Literal("1", "1"), Plus, Call(Identifier("foo"), [Literal("20", "20")])))])
(5) | Call(Identifier("log"), [Literal("--------", "\"--------\"")])
(6) | Call(Identifier("log"), [Identifier("t")])
/*-----tree-end------*/
 log => [String("0 |")]
 log => [String("1 |")]
 log => [String("2 |")]
 log => [String("3 |")]
 log => [String("4 |")]
 log => [String("5 |")]
 log => [String("6 |")]
 log => [String("7 |")]
 log => [String("8 |")]
 log => [String("9 |")]
 log => [String("--------")]
 log => [String("0 |")]
 log => [String("1 |")]
 log => [String("2 |")]
 log => [String("3 |")]
 log => [String("4 |")]
 log => [String("5 |")]
 log => [String("6 |")]
 log => [String("7 |")]
 log => [String("8 |")]
 log => [String("9 |")]
 log => [String("10 |")]
 log => [String("11 |")]
 log => [String("12 |")]
 log => [String("13 |")]
 log => [String("14 |")]
 log => [String("15 |")]
 log => [String("16 |")]
 log => [String("17 |")]
 log => [String("18 |")]
 log => [String("19 |")]
 log => [String("--------")]
 log => [Int(21)]
解析耗时: 9518µs (9ms)
```