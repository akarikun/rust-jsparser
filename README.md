### Rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
 fn main() -> Result<(), String> {
    let input = r#"
    log(a+1);
    log(a+1+a*2-a);
    log("a"+"1");
    log("a"+2+3);
    log(2+3);
    log(2+3+"a");
    if(a==1){ log("[1]:a==1"); } else{ log("[1]:a!=1"); }
    if(a==12){ log("[2]:a==12"); } else{ log("[2]:a!=1"); }
    log(foo2(1,2,3));
    function foo(a,b,c){return a+b+c;}
    function foo2(b,c){return a+b+c;}
    log(foo(1,2,3));
    log(foo2(2,3));
    // log(foo3(1,2,3));//执行到这里报错后不执行后面的语句
    log(a+1);
    log(add(100,200));
    log(add(add(1,2),add(3,4,5)));
"#;
    let start = Instant::now();
    let mut lexer = Lexer::new(String::from(input));
    lexer.print(); //打印token
    let mut parser = Parser::new(Box::new(lexer));

    let mut program = parser.parse_program()?;
    program.print_tree(); //打印树
    program.bind_value(String::from("a"), JSType::Int(12));
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
<log> <(> <a> <+> <1> <)> <;>
<log> <(> <a> <+> <1> <+> <a> <*> <2> <-> <a> <)> <;>
<log> <(> <a> <+> <1> <)> <;>
<log> <(> <a> <+> <2> <+> <3> <)> <;>
<log> <(> <2> <+> <3> <)> <;>
<log> <(> <2> <+> <3> <+> <a> <)> <;>
<key:if> <(> <a> <==> <1> <)> <{> <log> <(> <[1]:a==1> <)> <;> <}> <key:else> <{> <log> <(> <[1]:a!=1> <)> <;> <}>
<key:if> <(> <a> <==> <12> <)> <{> <log> <(> <[2]:a==12> <)> <;> <}> <key:else> <{> <log> <(> <[2]:a!=1> <)> <;> <}>
<log> <(> <foo2> <(> <1> <,> <2> <,> <3> <)> <)> <;>
<key:function> <foo> <(> <a> <,> <b> <,> <c> <)> <{> <key:return> <a> <+> <b> <+> <c> <;> <}>
<key:function> <foo2> <(> <b> <,> <c> <)> <{> <key:return> <a> <+> <b> <+> <c> <;> <}>
<log> <(> <foo> <(> <1> <,> <2> <,> <3> <)> <)> <;>
<log> <(> <foo2> <(> <2> <,> <3> <)> <)> <;>
<log> <(> <a> <+> <1> <)> <;>
<log> <(> <add> <(> <100> <,> <200> <)> <)> <;>
<log> <(> <add> <(> <add> <(> <1> <,> <2> <)> <,> <add> <(> <3> <,> <4> <,> <5> <)> <)> <)> <;>
/*-------- end --------*/
(1) | Call(Identifier("log"), [Infix(Identifier("a"), Plus, Literal("1"))])
(2) | Call(Identifier("log"), [Infix(Infix(Infix(Identifier("a"), Plus, Literal("1")), Plus, Infix(Identifier("a"), Multiply, Literal("2"))), Subtract, Identifier("a"))])
(3) | Call(Identifier("log"), [Infix(Literal("a"), Plus, Literal("1"))])
(4) | Call(Identifier("log"), [Infix(Infix(Literal("a"), Plus, Literal("2")), Plus, Literal("3"))])
(5) | Call(Identifier("log"), [Infix(Literal("2"), Plus, Literal("3"))])
(6) | Call(Identifier("log"), [Infix(Infix(Literal("2"), Plus, Literal("3")), Plus, Literal("a"))])
(7) | If(Infix(Identifier("a"), Equal, Literal("1")), BlockStatement([Call(Identifier("log"), [Literal("[1]:a==1")])]), BlockStatement([Call(Identifier("log"), [Literal("[1]:a!=1")])]))
(8) | If(Infix(Identifier("a"), Equal, Literal("12")), BlockStatement([Call(Identifier("log"), [Literal("[2]:a==12")])]), BlockStatement([Call(Identifier("log"), [Literal("[2]:a!=1")])]))
(9) | Call(Identifier("log"), [Call(Identifier("foo2"), [Literal("1"), Literal("2"), Literal("3")])])
(10) | Function(Identifier("foo"), [Identifier("a"), Identifier("b"), Identifier("c")], BlockStatement([Return(Infix(Infix(Identifier("a"), Plus, Identifier("b")), Plus, Identifier("c")))]))
(11) | Function(Identifier("foo2"), [Identifier("b"), Identifier("c")], BlockStatement([Return(Infix(Infix(Identifier("a"), Plus, Identifier("b")), Plus, Identifier("c")))]))
(12) | Call(Identifier("log"), [Call(Identifier("foo"), [Literal("1"), Literal("2"), Literal("3")])])
(13) | Call(Identifier("log"), [Call(Identifier("foo2"), [Literal("2"), Literal("3")])])
(14) | Call(Identifier("log"), [Infix(Identifier("a"), Plus, Literal("1"))])
(15) | Call(Identifier("log"), [Call(Identifier("add"), [Literal("100"), Literal("200")])])
(16) | Call(Identifier("log"), [Call(Identifier("add"), [Call(Identifier("add"), [Literal("1"), Literal("2")]), Call(Identifier("add"), [Literal("3"), Literal("4"), Literal("5")])])])
 log => [Int(13)]
 log => [Int(25)]
 log => [String("a1")]
 log => [String("a23")]
 log => [Int(5)]
 log => [String("5a")]
 log => [String("[1]:a!=1")]
 log => [String("[2]:a==12")]
 log => [Int(15)]
 log => [Int(6)]
 log => [Int(17)]
 log => [Int(13)]
 log => [Int(300)]
 log => [Int(15)]
解析耗时: 9256µs (9ms)
```