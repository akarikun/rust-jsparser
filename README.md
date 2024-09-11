### rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
    let input = r#"
    1+1+d+b                                         //Y
    1+1                                             //Y 在考虑是否禁止"非;换行"的逻辑
    +d+b                                            //Y
    a(1+1+d+b);                                     //Y
    let aa = a+(b-c) || (c&&d) &&e || (f*e);        //Y
    a+b&&c+d;                                       //Y
    let aabbb = a+b&&c+d;                           //Y
    let t=a+b+c;                                    //Y
    (a+b);                                          //Y
    a+(b*(c-d));                                    //Y
    a+(b*(c-d));                                    //Y
    let x = 11+22*(33+44)-55;                       //Y
    let y = a+(b*(c-d));                            //Y
    a();                                            //Y
    a(b());                                         //Y
    a(b(),c(),d(a+(b*(c-d))));                      //Y
    x++;                                            //Y
    a==b;                                           //Y
    c&&d;                                           //Y
    a==b&&c;                                        //Y
    c&&d;                                           //Y
    ;                                               //Y
    a;                                              //Y
    b                                               //Y
    c                                               //Y 
"#;
<1> <+> <1> <+> <d> <+>
<b> <1> <+> <1>
<+> <d> <+> <b>
<a> <(> <1> <+> <1> <+> <d> <+> <b> <)> <;>
<key:let> <aa> <=> <a> <+> <(> <b> <-> <c> <)> <||> <(> <c> <&&> <d> <)> <&&> <e> <||> <(> <f> <*> <e> <)> <;>
<a> <+> <b> <&&> <c> <+> <d> <;>
<key:let> <aabbb> <=> <a> <+> <b> <&&> <c> <+> <d> <;>
<key:let> <t> <=> <a> <+> <b> <+> <c> <;>
<(> <a> <+> <b> <)> <;>
<a> <+> <(> <b> <*> <(> <c> <-> <d> <)> <)> <;>
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
<c>
/*-------- end --------/*
eval LEN:23
eval stmt =>  (1) Expression(Infix(Infix(Infix(Number(1), Plus, Number(1)), Plus, Identifier("d")), Plus, Identifier("b")))   
eval stmt =>  (2) Expression(Infix(Infix(Infix(Number(1), Plus, Number(1)), Plus, Identifier("d")), Plus, Identifier("b")))   
eval stmt =>  (3) Expression(Call(Identifier("a"), [Infix(Infix(Infix(Number(1), Plus, Number(1)), Plus, Identifier("d")), Plus, Identifier("b"))]))
eval stmt =>  (4) Variable("let", "aa", Infix(Infix(Infix(Infix(Identifier("a"), Plus, Infix(Identifier("b"), Minus, Identifier("c"))), Or, Infix(Identifier("c"), And, Identifier("d"))), And, Identifier("e")), Or, Infix(Identifier("f"), Multiply, Identifier("e"))))
eval stmt =>  (5) Expression(Infix(Infix(Identifier("a"), Plus, Identifier("b")), And, Infix(Identifier("c"), Plus, Identifier("d"))))
eval stmt =>  (6) Variable("let", "aabbb", Infix(Infix(Identifier("a"), Plus, Identifier("b")), And, Infix(Identifier("c"), Plus, Identifier("d"))))
eval stmt =>  (7) Variable("let", "t", Infix(Infix(Identifier("a"), Plus, Identifier("b")), Plus, Identifier("c")))
eval stmt =>  (8) Expression(Infix(Identifier("a"), Plus, Identifier("b")))
eval stmt =>  (9) Expression(Infix(Identifier("a"), Plus, Infix(Identifier("b"), Multiply, Infix(Identifier("c"), Minus, Identifier("d")))))
eval stmt =>  (10) Expression(Infix(Identifier("a"), Plus, Infix(Identifier("b"), Multiply, Infix(Identifier("c"), Minus, Identifier("d")))))
eval stmt =>  (11) Variable("let", "x", Infix(Infix(Number(11), Plus, Infix(Number(22), Multiply, Infix(Number(33), Plus, Number(44)))), Minus, Number(55)))
eval stmt =>  (12) Variable("let", "y", Infix(Identifier("a"), Plus, Infix(Identifier("b"), Multiply, Infix(Identifier("c"), Minus, Identifier("d")))))
eval stmt =>  (13) Expression(Call(Identifier("a"), []))
eval stmt =>  (14) Expression(Call(Identifier("a"), [Call(Identifier("b"), [])]))
eval stmt =>  (15) Expression(Call(Identifier("a"), [Call(Identifier("b"), []), Call(Identifier("c"), []), Call(Identifier("d"), [Infix(Identifier("a"), Plus, Infix(Identifier("b"), Multiply, Infix(Identifier("c"), Minus, Identifier("d"))))])]))       
eval stmt =>  (16) Expression(Update(Identifier("x"), INC, false))
eval stmt =>  (17) Expression(Infix(Identifier("a"), Equal, Identifier("b")))
eval stmt =>  (18) Expression(Infix(Identifier("c"), And, Identifier("d")))
eval stmt =>  (19) Expression(Infix(Infix(Identifier("a"), Equal, Identifier("b")), And, Identifier("c")))
eval stmt =>  (20) Expression(Infix(Identifier("c"), And, Identifier("d")))
eval stmt =>  (21) Expression(Identifier("a"))
eval stmt =>  (22) Expression(Identifier("b"))
eval stmt =>  (23) Expression(Identifier("c"))