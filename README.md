### rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
let input = r#"
    // let y = 11+(22*(33-44));  
    let x = 11+22*(33+44)-55;
    x++;
    a==b;
    c&&d;
    a==b&&c;
"#;

/*--------print--------*/
1:<key:let> 2:<x> 3:<=> 4:<11> 5:<+> 6:<22> 7:<*> 8:<(> 9:<33> 10:<+> 11:<44> 12:<)> 13:<-> 14:<55> 15:<;> 16:<x> 17:<++> 18:<;> 19:<a> 20:<==> 21:<b> 22:<;> 23:<c> 24:<&&> 25:<d> 26:<;> 27:<a> 28:<==> 29:<b> 30:<&&> 31:<c> 32:<;>
/*-------- end --------/*
LEN:5
eval stmt =>  Variable("let", "x", Infix(Infix(Number(11), Plus, Infix(Number(22), Multiply, Infix(Number(33), Plus, Number(44)))), Minus, Number(55)))     
eval stmt =>  Expression(Update(Identifier("x"), INC, false))
eval stmt =>  Expression(Infix(Identifier("a"), Equal, Identifier("b")))
eval stmt =>  Expression(Infix(Identifier("c"), And, Identifier("d")))
eval stmt =>  Expression(Infix(Infix(Identifier("a"), Equal, Identifier("b")), And, Identifier("c")))
```
