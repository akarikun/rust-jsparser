### Rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

###### 完成如下:
```
fn main() -> Result<(), String> {
    let input = r#"
    log(add(add(1,2),add(3,4,5)));
    test(11);
    function test(val){
        for(let i = 0;i<10;i++){
            log("test:"+(i+val+a));
        }
    }
    log(foo3(1,2,3));//执行到这里报错后不执行后面的语句
    test(11);
"#;
    let start = Instant::now();
    let mut lexer = Lexer::new(String::from(input));
    // lexer.print(); //打印token
    let mut parser = Parser::new(Box::new(lexer));

    let mut program = parser.parse_program()?;
    // program.print_tree(); //打印树
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
 log => [Int(15)]
 log => [String("test:23")]
 log => [String("test:24")]
 log => [String("test:25")]
 log => [String("test:26")]
 log => [String("test:27")]
 log => [String("test:28")]
 log => [String("test:29")]
 log => [String("test:30")]
 log => [String("test:31")]
 log => [String("test:32")]
Uncaught ReferenceError: foo3 is not defined
解析耗时: 5540µs (5ms)
```