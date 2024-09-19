### Rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
 let input = r#"
    log(a+1);
    log(a+1+a*2-a);
    log("a"+"1");
    log("a"+2+3);
    log(2+3);
    log(2+3+"a");
    if(a==1){ log(1); } else{ log(2); }
    if(a==100){ log(3); } else{ log(4); }
    function foo(a,b,c){return a+b+c;}
    function foo2(b,c){return a+b+c;}
    log(foo(1,2,3));
    log(foo2(2,3));
"#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(Box::new(lexer));
    let mut program = parser.parse_program();
    program.bind_global_value(String::from("a"), JSType::Int(100));
    program.register_method(
        String::from("log"),
        Box::new(|args| {
            println!("register_method:log=> {:?}", args);
        }),
    );
    program.run();
```
```
/*--------print--------*/
<log> <(> <a> <+> <1> <)> <;>
<log> <(> <a> <+> <1> <+> <a> <*> <2> <-> <a> <)> <;>
<log> <(> <a> <+> <1> <)> <;>
<log> <(> <a> <+> <2> <+> <3> <)> <;>
<log> <(> <2> <+> <3> <)> <;>
<log> <(> <2> <+> <3> <+> <a> <)> <;>
<key:if> <(> <a> <==> <1> <)> <{> <log> <(> <1> <)> <;> <}> <key:else> <{> <log> <(> <2> <)> <;> <}>
<key:if> <(> <a> <==> <100> <)> <{> <log> <(> <3> <)> <;> <}> <key:else> <{> <log> <(> <4> <)> <;> <}>
<key:function> <foo> <(> <a> <,> <b> <,> <c> <)> <{> <key:return> <a> <+> <b> <+> <c> <;> <}>    
<key:function> <foo2> <(> <b> <,> <c> <)> <{> <key:return> <a> <+> <b> <+> <c> <;> <}>
<log> <(> <foo> <(> <1> <,> <2> <,> <3> <)> <)> <;>
<log> <(> <foo2> <(> <2> <,> <3> <)> <)> <;>
/*-------- end --------*/
register_method:log=> [Int(101)]
register_method:log=> [Int(201)]
register_method:log=> [String("a1")]
register_method:log=> [String("a23")]
register_method:log=> [Int(5)]
register_method:log=> [String("5a")]
register_method:log=> [Int(2)]
register_method:log=> [Int(3)]
register_method:log=> [Int(6)]
register_method:log=> [Int(105)]

```