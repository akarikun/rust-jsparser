mod jsparser;
use jsparser::utility::run_console;

fn main() -> Result<(), String> {
    _ = run_console(
        r#"
        for(let i=0;i<10;i++){
            if(i>=3){
                break;
            }
            log("第"+(i+1)+"次调用ajax");
            ajax({
                url:'https://ipinfo.io',
                type:'get',
                success:function(e){
                    log(e);
                }
            });
        }
"#
        .into(),
    );

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log() {
        _ = run_console("log(1);".into());
    }

    #[test]
    fn test_fn1() {
        let _ = run_console(
            r#"
    function foo(a){
        for(let i=0;i<a;i++)
            log(i+" |");
        //log(i);                  //Uncaught ReferenceError: i is not defined
        return a;
    }
    //log(i);                      //Uncaught ReferenceError: i is not defined
"#
            .to_string(),
        );
    }
    #[test]
    fn test_fn2() {
        let _ = run_console(
            r#"
            let i = 0;
            function foo(){
                for(;;i++){
                    if(i<10)
                        log(i);
                    else
                        return;
                }
                log(123);
            }
            foo();
        "#
            .to_string(),
        );
    }
    #[test]
    fn test_for() {
        let _ = run_console(
            r#"
            for(let i = 0;i<10;i++){
                log(i);
            }
        "#
            .to_string(),
        );

        let _ = run_console(
            r#"
                let i = 0;
                for(;;){
                    if(i<10){
                        log(i);
                    }
                    else{
                        break;
                    }
                    i++;
                }
            "#
            .to_string(),
        );

        let _ = run_console(
            r#"
            let i = 0;
            for(;;i++;){
                if(i<10)
                    log(i);
                else
                    break;
            }
        "#
            .to_string(),
        );
    }

    #[test]
    fn test_json() {
        let _ = run_console(
            r#"
            let json = {[1+1]:2} //[a+1]:5 暂未实现
        "#
            .to_string(),
        );
    }
}