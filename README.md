### Rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
    let input = r#"
        print(1*2*3-4/2);
        if(1==2){ print(1); } else{ print(2); }
        if(1==1){ print(3); } else{ print(4); }
"#;
    let mut lexer = Lexer::new(String::from(input));
    lexer.print();

    let mut parser = Parser::new(Box::new(lexer));
    let mut program = parser.parse_program();
    program.register_method(
        String::from("print"),
        Box::new(|args| {
            println!("register_method:print=> {:?}", args);
        }),
    );
    program.eval(false,Vec::new());
```
```
/*--------print--------*/
<print> <(> <1> <*> <2> <*> <3> <-> <4> </> <2> <)> <;>
<key:if> <(> <1> <==> <2> <)> <{> <print> <(> <1> <)> <;> <}> <key:else> <{> <print> <(> <2> <)> <;> <}>
<key:if> <(> <1> <==> <1> <)> <{> <print> <(> <3> <)> <;> <}> <key:else> <{> <print> <(> <4> <)> <;> <}>
/*-------- end --------*/
register_method:print=> [Float(4.0)]
register_method:print=> [Float(2.0)]
register_method:print=> [Float(3.0)]
```