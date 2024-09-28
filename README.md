### Rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)


###### 当前程序用到current_token(cur)以及peek_token(peek)表示当前跟下个token
###### 由于我最开始是两个token各种match实现的，到后面写到解析树时用递归实现想到解析token也可以用递归,然后搜了一下，果然。。。

解析token
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
    log(add(add(1,2,3),add(4,5)));
    log(test(11)(22));
    function test(val){
        for(let i = 0;i<10;i++){
            if (i%2==0)
                log("test:"+i+" "+(i+val+a)) 
            else
                log("test:"+i+" "+(i-val-a));
        }
        return function(abc){
            return val+abc+a;
        }
    }
    log("------");
    log(val);//val is not defined 
    test(22);
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
<log> <(> <add> <(> <add> <(> <1> <,> <2> <,> <3> <)> <,> <add> <(> <4> <,> <5> <)> <)> <)> <;>
<log> <(> <test> <(> <11> <)> <(> <22> <)> <)> <;>
<key:function> <test> <(> <val> <)> <{>
<key:for> <(> <key:let> <i> <=> <0> <;> <i> <<> <10> <;> <i> <++> <)> <{>
<key:if> <(> <i> <%> <2> <==> <0> <)>
<log> <(> <"test:"> <+> <i> <+> <" "> <+> <(> <i> <+> <val> <+> <a> <)> <)>
<key:else>
<log> <(> <"test:"> <+> <i> <+> <" "> <+> <(> <i> <-> <val> <-> <a> <)> <)> <;>
<}>
<key:return> <key:function> <(> <abc> <)> <{>
<key:return> <val> <+> <abc> <+> <a> <;> 
<}>
<}>
<log> <(> <"------"> <)> <;>
<log> <(> <val> <)> <;>
<test> <(> <22> <)> <;>
/*-------- end --------*/
/*--------tree--------*/
(1) | Call(Identifier("log"), [Call(Identifier("add"), [Call(Identifier("add"), [Literal("1", "1"), Literal("2", "2"), Literal("3", "3")]), Call(Identifier("add"), [Literal("4", "4"), Literal("5", "5")])])])
(2) | Call(Identifier("log"), [Call(Call(Identifier("test"), [Literal("11", "11")]), [Literal("22", "22")])])
(3) | Function(Identifier("test"), [Identifier("val")], BlockStatement([For(Variable2([(Let, "i", Literal("0", "0"))]), Infix(Identifier("i"), LT, Literal("10", "10")), Update(Identifier("i"), "++", false), BlockStatement([If(Infix(Infix(Identifier("i"), Modulo, Literal("2", "2")), Equal, Literal("0", "0")), Call(Identifier("log"), [Infix(Infix(Infix(Literal("test:", "\"test:\""), Plus, Identifier("i")), Plus, Literal(" ", "\" \"")), Plus, Infix(Infix(Identifier("i"), Plus, Identifier("val")), Plus, Identifier("a")))]), Call(Identifier("log"), [Infix(Infix(Infix(Literal("test:", "\"test:\""), Plus, Identifier("i")), Plus, Literal(" ", "\" \"")), Plus, Infix(Infix(Identifier("i"), Subtract, Identifier("val")), Subtract, Identifier("a")))]))])), Return(Function(Empty, [Identifier("abc")], BlockStatement([Return(Infix(Infix(Identifier("val"), Plus, Identifier("abc")), Plus, Identifier("a")))])))]))
(4) | Call(Identifier("log"), [Literal("------", "\"------\"")])
(5) | Call(Identifier("log"), [Identifier("val")])
(6) | Call(Identifier("test"), [Literal("22", "22")])
/*-----tree-end------*/
 log => [Int(15)]
 log => [String("test:0, 23")]
 log => [String("test:1, -22")]
 log => [String("test:2, 25")]
 log => [String("test:3, -20")]
 log => [String("test:4, 27")]
 log => [String("test:5, -18")]
 log => [String("test:6, 29")]
 log => [String("test:7, -16")]
 log => [String("test:8, 31")]
 log => [String("test:9, -14")]
 log => [Int(45)]
 log => [String("------")]
"Uncaught ReferenceError: val is not defined"
解析耗时: 5396µs (5ms)
```