### Rust实现JS解析

ast树参考 [parse.html](https://esprima.org/demo/parse.html)

```
# lib.rs 生成 webassembly (注意生成wasm时,有的库会不支持,如要生成请移除不支持的库)

# 安装所需工具
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
npm i wasm-opt -g

# 编译wasm
cargo build --target wasm32-unknown-unknown --release --lib

wasm-opt -Oz -o target/wasm32-unknown-unknown/release/jsparser.wasm target/wasm32-unknown-unknown/release/jsparser.wasm #压缩

wasm-bindgen target/wasm32-unknown-unknown/release/jsparser.wasm --out-dir ./pkg --web

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

### 效果如下:
![708b5212-8875-4c1f-9e0b-1376d513e346.png](https://raw.githubusercontent.com/akarikun/rust-jsparser/master/images/708b5212-8875-4c1f-9e0b-1376d513e346.png)