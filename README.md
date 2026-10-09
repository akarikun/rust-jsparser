### Rust实现JS解析

#### 学习Rust

ast参考 [parse.html](https://esprima.org/demo/parse.html)

```
# lib.rs 生成 webassembly 

# 安装所需工具
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
npm i wasm-opt -g

# 编译wasm及压缩(最终生成在pkg目录)
cargo build --target wasm32-unknown-unknown --release --lib && wasm-opt -Oz -o target/wasm32-unknown-unknown/release/jsparser.wasm target/wasm32-unknown-unknown/release/jsparser.wasm && wasm-bindgen target/wasm32-unknown-unknown/release/jsparser.wasm --out-dir ./pkg --web
```
##### pkg/index.html
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>jsparser</title>
</head>
<body>
  <script type="module">
    import init, { run_code } from './jsparser.js';

    ~(async()=>{
        await init();
        run_code(`
            for(let i=0;i<10;i++){
                log(i)
            }
      `)
    })();
  </script>
</body>
</html>
```

### 基础解析结构

- `parse_statement` 处理变量声明、控制语句、代码块和语句结束。
- `parse_expression(min_precedence)` 使用 precedence climbing 递归构建表达式；运算符的优先级和结合性只在 `infix` 中定义。
- `parse_prefix` 处理字面量、标识符、一元运算和括号；`parse_postfix` 处理调用、成员访问及后缀更新。
- Lexer 顺序生成 Token，使用循环跳过注释；字符串与数字分别保留类型。语法错误返回带行列信息的 `Err`。

解释器仍只实现部分 AST 的执行。逻辑/位运算、成员访问等解析通过，不代表执行功能已完成；严格相等、稀疏数组和完整模板字符串语法尚未支持。`cargo test --offline` 可运行本地测试，真实网络 AJAX 测试默认忽略。
