use std::fs;
use std::env;
use std::io::{Write, BufRead};

mod interpreter;
use interpreter::State;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut main_state = State::new();

    if args.len() > 1 {
        let input = fs::File::open(&args[1]).expect("File not found!");
        let reader = std::io::BufReader::new(input);

        for line in reader.lines() {
            interpreter::interpret(line.unwrap().as_str(), &mut main_state);
        }

        for v in &main_state.variables {
            println!("Name: {}\nValue: {:?}\n\n", v.0, v.1);
        }
    } else {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        loop {
            print!(">>> ");
            stdout.flush().unwrap();
            let mut input = String::new();
            interpreter::interpret(match stdin.read_line(&mut input) {
                Ok(_) => input.as_str(),
                Err(text) => panic!("{}", text)
            }, &mut main_state);

            for v in &main_state.variables {
                println!("Name: {}\nValue: {:?}\n\n", v.0, v.1);
            }
        }
    }
}
