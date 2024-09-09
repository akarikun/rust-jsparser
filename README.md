### rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
let input = r#"
    (a+b);                      //Y
    a+(b*(c-d));                //Y
    //a+(b*(c-d));              //Y
    let x = 11+22*(33+44)-55;   //Y
    let y = a+(b*(c-d));        //Y
    a();                        //Y
    a(b());                     //Y
    a(b(),c(),d(a+(b*(c-d))));  //Y
    x++;                        //Y
    a==b;                       //Y
    c&&d;                       //Y
    a==b&&c;                    //Y
    c&&d;                       //Y
    ;                           //Y
    a;                          //Y
    b                           //Y
"#;
/*--------print--------*/
<(> <a> <+> <b> <)> <;>
<a> <+> <(> <b> <*> <(> <c> <-> <d> <)> <)> <;> 
<key:let> <x> <=> <11> <+> <22> <*> <(> <33> <+> <44> <)> <-> <55> <;>
<key:let> <y> <=> <a> <+> <(> <b> <*> <(> <c> <-> <d> <)> <)> <;>
<a> <(> <)> <;>
<a> <(> <b> <(> <)> <)> <;>
<a> <(> <b> <(> <)> <,> <c> <(> <)> <,> <d> <(> <a> <+> <(> <b> <*> <(> <c> <-> <d> <)> <)> <)> <)> <;>
<x> <++> <;>
<a> <==> <b> <;>
<c> <&&> <d> <;>
<a> <==> <b> <&&> <c> <;>
<c> <&&> <d> <;> 
<;>
<a> <;>
<b>
/*-------- end --------/*
eval LEN:14
eval stmt =>  (1) Expression(Infix(Identifier("a"), Plus, Identifier("b")))
eval stmt =>  (2) Expression(Infix(Identifier("a"), Plus, Infix(Identifier("b"), Multiply, Infix(Identifier("c"), Minus, Identifier("d")))))
eval stmt =>  (3) Variable("let", "x", Infix(Infix(Number(11), Plus, Infix(Number(22), Multiply, Infix(Number(33), Plus, Number(44)))), Minus, Number(55)))
eval stmt =>  (4) Variable("let", "y", Infix(Identifier("a"), Plus, Infix(Identifier("b"), Multiply, Infix(Identifier("c"), Minus, Identifier("d")))))
eval stmt =>  (5) Expression(Call(Identifier("a"), []))
eval stmt =>  (6) Expression(Call(Identifier("a"), [Call(Identifier("b"), [])]))
eval stmt =>  (7) Expression(Call(Identifier("a"), [Call(Identifier("b"), []), Call(Identifier("c"), []), Call(Identifier("d"), [Infix(Identifier("a"), Plus, Infix(Identifier("b"), Multiply, Infix(Identifier("c"), Minus, Identifier("d"))))])]))
eval stmt =>  (8) Expression(Update(Identifier("x"), INC, false))
eval stmt =>  (9) Expression(Infix(Identifier("a"), Equal, Identifier("b")))
eval stmt =>  (10) Expression(Infix(Identifier("c"), And, Identifier("d")))
eval stmt =>  (11) Expression(Infix(Infix(Identifier("a"), Equal, Identifier("b")), And, Identifier("c")))
eval stmt =>  (12) Expression(Infix(Identifier("c"), And, Identifier("d")))
eval stmt =>  (13) Expression(Identifier("a"))
eval stmt =>  (14) Expression(Identifier("b"))
```
