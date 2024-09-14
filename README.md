### Rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
    let input = r#"
        let a = !b(1-m*n)/c&&a+2>=0||1<(-c)&&-p[-d-+c];
        if(a) alert(a);
        else if(b) c=d
        else if(c) c=d; else {
            let a=1,b,c=d;
        }
    "#;
/*--------print--------*/
<key:let> <a> <=> <!> <b> <(> <1> <-> <m> <*> <n> <)> </> <c> <&&> <a> <+> <2> <>=> <0> <||> <1> <<> <(> <-> <c> <)> <&&> <-> <p> <[> <-> <d> <-> <+> <c> <]> <;>
<key:if> <(> <a> <)> <alert> <(> <a> <)> <;>
<key:else> <key:if> <(> <b> <)> <c> <=> <d>
<key:else> <key:if> <(> <c> <)> <c> <=> <d> <;> <key:else> <{>
<key:let> <a> <=> <1> <,> <b> <,> <c> <=> <d> <;>
<}>
/*-------- end --------*/
eval LEN:2
eval expr =>  (1) Variable("let", "a", Infix(Infix(Infix(Prefix(Not, Call("b", [Infix(Number(1), Minus, Infix(Identifier("m"), Multiply, Identifier("n")))])), Divide, Identifier("c")), And, Infix(Infix(Identifier("a"), Plus, Number(2)), GTE, Number(0))), Or, Infix(Infix(Number(1), LT, Prefix(Negate, Identifier("c"))), And, Prefix(Negate, Member("p", [Infix(Prefix(Negate, Identifier("d")), Minus, Prefix(Abs, Identifier("c")))])))))
eval expr =>  (2) If(Identifier("a"), Expression(Call("alert", [Identifier("a")])), If(Identifier("b"), Expression(Assignment("c", Identifier("d"))), If(Identifier("c"), Expression(Assignment("c", Identifier("d"))), BlockStatement([Variable("let", "a", Number(1)), Variable("let", "b", Empty), Variable("let", "c", Identifier("d"))]))))   

```