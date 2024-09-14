### Rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)


|len|step1|expr|desc|
|-|-|-|-|
|3|let a = 1| < key: let > < ident > < ptor: = >|[0]=let <br /> [1]=< step2 >|
|2|if(a==b){}| < key: if > < ptor: ( >| [0]=if <br />[1]=(<br />[2]=< step2 > |
|...||||

|len|step2|expr|desc|
|-|-|-|-|
|2|a = b| < ident > < = > b | a = < base >|

|len|base|expr|desc|
|-|-|-|-|
|2|!a ; ~a; +a ; -a ; ~a ; ++a ; --a ;  a[ ; a( ;...| < ptor: (slot) > < ident > |
|2|!1 ; +1 ; -1 ; ~1;| < ptor: (slot) > < num >|
|2|a++; a--;|< ident > < ptor: (slot) >|
||+-~!a|< ptor: (slot) > < ptor: (slot) > ...| ptor 可无限套娃
|-||||
|| a + b |  < base > < ptor: (slot) > < base >| (ptor ; base) 可无限套娃|
