### rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
    let input = r#"
        let a = !b[1*m-n]+c-d*e+-f*g(2*3-h) &&aa+2>=0||1<(bb*3-cc)&&abc-p(bbb-333+ccc);
    "#;
/*--------print--------*/
<key:let> <a> <=> <!> <b> <[> <1> <*> <m> <-> <n> <]> <+> <c> <-> <d> <*> <e> <+> <-> <f> <*> <g> <(> <2> <*> <3> <-> <h> <)> <&&> <aa> <+> <2> <>=> <0> <||> <1> <<> <(> <bb> <*> <3> <-> <cc> <)> <&&> <abc> <-> <p> <(> <bbb> <-> <333> <+> <ccc> <)> <;>
/*-------- end --------/*
eval LEN:1
eval stmt =>  (1) Variable("let", "a", Infix(Infix(Infix(Infix(Infix(Prefix(Not, Member(Identifier("b"), [Infix(Infix(Number(1), Multiply, Identifier("m")), Minus, Identifier("n"))])), Plus, Identifier("c")), Minus, Infix(Identifier("d"), Multiply, Identifier("e"))), Plus, Infix(Prefix(Negate, Identifier("f")), Multiply, Call(Identifier("g"), [Infix(Infix(Number(2), Multiply, Number(3)), Minus, Identifier("h"))]))), And, Infix(Infix(Identifier("aa"), Plus, Number(2)), GTE, Number(0))), Or, Infix(Infix(Number(1), LT, Infix(Infix(Identifier("bb"), Multiply, Number(3)), Minus, Identifier("cc"))), And, Infix(Identifier("abc"), Minus, Call(Identifier("p"), [Infix(Infix(Identifier("bbb"), Minus, Number(333)), Plus, Identifier("ccc"))])))))