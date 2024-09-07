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

<key:let> <x> <=> <11> <+> <22> <*> <(> <33> <+> <44> <)> <-> <55> <;>
<x> <++> <;>
<a> <==> <b> <;>
<c> <&&> <d> <;>
<a> <==> <b> <&&> <c> <;>
/*-------- end --------/*
LEN:5
calc =>  let x = 1650
eval stmt =>  Expression(Update(Identifier("x"), INC, false))
eval stmt =>  Expression(Infix(Identifier("a"), Equal, Identifier("b")))
eval stmt =>  Expression(Infix(Identifier("c"), And, Identifier("d")))
eval stmt =>  Expression(Infix(Infix(Identifier("a"), Equal, Identifier("b")), And, Identifier("c")))
```

```
pub enum Expr {
    Empty,                                              //base
    Identifier(String),
    Number(i64),

    Prefix(Prefix, Box<Expr>),                          // !a  -1
    Call(Box<Expr>, Vec<Expr>),                         //Box<Expr> => Identifier(String)
    Infix(Box<Expr>, Infix, Box<Expr>),                 // a+b   算术可替换  +,-,*,/
    Logical(Box<Expr>,Logical,Box<Expr>),               // a&&b  逻辑可替换  &&,||,!
    Expression(Box<Expr>,TokenPunctuator,Expression),   //这个可能会优化掉
}

Expr:
    ;          //empty
    a          //ident
    1          //num
    call(Expr)
    a[]
    a+b        //Infix
    a==b       //Infix
    
Stmt: 
    Expr
    let a=Expr
    if(Expr){}
```
###### ==黄色==表示可换成其他Expr

|序号|表达式|步骤|生成|
|-|-|-|-|
|$1(base)|a+b|(@1 a+b)|Infix(a,+,b)<br/>|
|$1(base)|a\====b==|(@1 a==b)|Infix(a,\==,b)<br/>|
|$2|a\====b==+==c==<br/>a== ==(\$1)==|(@2 a==(@1 b+c))|Infix(a,\==,<u>_**Infix**_</u>) <br/> &nbsp; <u>_**Infix(b,+,c)**_</u>|
|$3|a====b==&&==c==|(@2 (@1 a==b)&&c)|Infix(Infix(a,==,b),&&,c)|
|$4|a\==b&&c==d<br/> ==(\$1)== && ==(\$1)==|(@3 (@1 a\==b)&&(@2 c==d))| Infix(Infix(a,\==,b),&&,Infix(c,==,d))|