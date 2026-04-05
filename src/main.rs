use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

fn main() -> ExitCode {
    let input = match read_input() {
        Ok(input) => input,
        Err(err) => {
            eprintln!("md2html: {err}");
            return ExitCode::from(1);
        }
    };

    print!("{}", md2html::markdown_to_html(&input));
    ExitCode::SUCCESS
}

fn read_input() -> io::Result<String> {
    let mut args = env::args().skip(1);
    if let Some(path) = args.next() {
        fs::read_to_string(path)
    } else {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        Ok(input)
    }
}

