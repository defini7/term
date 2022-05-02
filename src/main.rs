use std::fs;
use std::env;

mod interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Here is no filename!");
    }

    match fs::read_to_string(&args[1]) {
        Ok(data) => {
            let exit_code = interpreter::interpret(&data);
            println!("Exited with code {:?}", exit_code);
        },
        Err(info) => panic!("{}", info)
    }
}
