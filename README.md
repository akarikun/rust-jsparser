### rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
let input = r#"
    //let y = 11+(22*(33-44));  
    let x = 11+22*(33+44)-55;
    x++;
    a==b;
    c&&d;
"#;

/*--------print--------*/
<key:let> <x> <=> <11> <+> <22> <*> <(> <33> <+> <44> <)> <-> <55> <;>
<x> <++> <;>
<a> <==> <b> <;>
<c> <&&> <d> <;>
/*-------- end --------/*
cur:[Punctuator(Semicolon),51],peek:[Ident("c"),52]
LEN:4
calc =>  let x = 1650
eval stmt =>  Expression(Expression(Identifier("x"), INC, Update))
eval stmt =>  Expression(Binary(Identifier("a"), Equal, Identifier("b")))
eval stmt =>  Expression(Logical(Identifier("c"), And, Identifier("d")))
```