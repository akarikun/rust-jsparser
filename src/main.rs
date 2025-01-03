mod jsparser;
use jsparser::utility::run_console;

fn main() -> Result<(), String> {
    _ = run_console(
        r#"
            for(let i=0;i<10;i++){
                if(i>=3){ break;}
                log("第"+(i+1)+"次调用ajax");
                ajax({
                    url:'http://ipinfo.io',
                    type:'get',
                    success:function(e){
                        // log("第"+i+"次调用ajax");// log => [String("第4次调用ajax")]
                        log(e);
                    }
                });
            }
    "#
        .to_owned(),
    );

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str() {
        _ = run_console(
            r#" 
            let a = `1${2+22}3${4}`; 
            log(a);
        "#
            .to_owned(),
        );
    }
    #[test]
    fn test_switch() {
        _ = run_console(
            r#"       
            let a =1;
            switch(a){
                case 1:
                default:break;
            }
    "#
            .to_owned(),
        );
        // _ = run_console(
        //     r#"
        //     let a =1;
        //     switch(a){
        //         case 2:{}break;
        //         default:
        //     }
        //     let d= 1;
        // "#
        //     .to_owned(),
        // );
    }

    #[test]
    fn test_ajax() {
        _ = run_console(
            r#"       
            for(let i=0;i<5;i++){
                if(i>=3){
                    break;
                }
                log("第"+(i+1)+"次调用ajax");
                ajax({
                    url:'https://ipinfo.io',
                    type:'get',
                    success:function(e){
                        // log("第"+i+"次调用ajax");// log => [String("第4次调用ajax")]
                        log(e);
                    }
                });
            }
    "#
            .to_owned(),
        );
    }

    #[test]
    fn test_log() {
        _ = run_console("log(1);".to_owned());
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
            .to_owned(),
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
            .to_owned(),
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
            .to_owned(),
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
            .to_owned(),
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
        _ = run_console(
            r#" 
        let b = 2;
        let json = {'a':1,b,c:3};// {[a+1]:5} 暂未实现该表达式
        log(json);
        "#
            .to_owned(),
        );
    }
    #[test]
    fn test_array() {
        _ = run_console(
            r#"
        let a = 123; 
        let arr = [1,2,3,a];
        log(arr);
        "#
            .to_owned(),
        );
    }
}
