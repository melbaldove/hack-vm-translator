use std::fs;
use vm_translator::{
    code_writer::CodeWriter,
    parser::{self, Command, Parser},
};

fn main() {
    let mut args = std::env::args();
    args.next();

    let file_name = args.next().unwrap_or_else(|| {
        eprintln!("ERROR: no filename supplied.");
        std::process::exit(1);
    });

    let file = fs::read_to_string(&file_name).unwrap_or_else(|err| {
        eprintln!("ERROR: {}: {}", file_name, err);
        std::process::exit(2);
    });

    let parser = Parser::build(&file).unwrap_or_else(|err| {
        eprintln!("ERROR: {}: {}", file_name, err);
        std::process::exit(3);
    });

    let file_name = file_name.strip_suffix(".vm").expect("Wrong file type");
    let mut code_writer = CodeWriter::build(&format!("{file_name}.asm")).unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        std::process::exit(3);
    });
    for command in parser {
        println!("{command}");
        match command {
            Command::ArithmeticLogical(_) => {
                code_writer.write_arithmetic(command);
            }
            Command::Push(_, _) | Command::Pop(_, _) => {
                code_writer.write_push_pop(command);
            }
            Command::Label(_) => todo!(),
            Command::Goto(_) => todo!(),
            Command::If(_) => todo!(),
            Command::Function(_) => todo!(),
            Command::Return() => todo!(),
            Command::Call(_) => todo!(),
        }
    }
}
