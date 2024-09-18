### Rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
let input = r#"
    print(1*2*3-4/2);
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
program.eval(true);
```
```
/*--------print--------*/
<print> <(> <1> <*> <2> <*> <3> <-> <4> </> <2> <)> <;>
/*-------- end --------*/
 eval expr => LEN:1
 eval expr => (1) Call(Identifier("print"), [Infix(Infix(Infix(Number(1.0), Multiply, Number(2.0)), Multiply, Number(3.0)), Minus, Infix(Number(4.0), Divide, Number(2.0)))])

register_method:print=> ["4"]
```