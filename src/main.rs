use std::{
    fs::{self},
    path::PathBuf,
};
use vm_translator::{
    code_writer::CodeWriter,
    parser::{Command, Parser},
};

fn main() {
    let mut args = std::env::args();
    args.next();

    let path = args
        .next()
        .map_or_else(|| PathBuf::from("."), |arg| PathBuf::from(arg));
    let file_stem = path.file_stem().and_then(|x| x.to_str()).unwrap();

    let asm_path = PathBuf::from(format!("./{file_stem}.asm"));
    let mut code_writer = CodeWriter::build(asm_path).unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        std::process::exit(3);
    });

    if path.is_dir() {
        let iter = path
            .read_dir()
            .expect("Expected to read_dir() successfully")
            .filter_map(|x| x.ok())
            .filter(|x| {
                x.path().is_file() && x.path().extension().and_then(|x| x.to_str()) == Some("vm")
            })
            .map(|x| x.path());

        for file_name in iter {
            translate_vm_code(file_name, &mut code_writer);
        }
    } else {
        translate_vm_code(path, &mut code_writer);
    }
}

fn translate_vm_code(file_name: PathBuf, code_writer: &mut CodeWriter) {
    let file_path = file_name.to_str().expect("Expected to_str() successfully");
    let file_name = file_name
        .file_name()
        .and_then(|x| x.to_str())
        .expect("Expected file_name() successfully");

    code_writer.set_file_name(String::from(file_name));

    println!("Translating {file_name}...");

    let file = fs::read_to_string(&file_path).unwrap_or_else(|err| {
        eprintln!("ERROR: {}: {}", file_path, err);
        std::process::exit(2);
    });

    let parser = Parser::build(&file).unwrap_or_else(|err| {
        eprintln!("ERROR: {}: {}", file_path, err);
        std::process::exit(3);
    });

    let mut current_function_name: Option<&str> = None;

    fn build_full_label(label: &str, current_function_name: Option<&str>) -> String {
        let mut full_label = String::new();
        if let Some(function_name) = current_function_name {
            full_label.push_str(&function_name);
        }
        full_label.push('$');
        full_label.push_str(label);
        full_label
    }

    for command in parser {
        println!("{command}");
        code_writer.write_comment(&command);
        match command {
            Command::ArithmeticLogical(_) => {
                code_writer.write_arithmetic(command);
            }
            Command::Push(_, _) | Command::Pop(_, _) => {
                code_writer.write_push_pop(command);
            }
            Command::Label(label) => {
                let full_label = build_full_label(label, current_function_name);
                code_writer.write_label(&full_label);
            }
            Command::Goto(label) => {
                let full_label = build_full_label(label, current_function_name);
                code_writer.write_goto(&full_label);
            }
            Command::If(label) => {
                let full_label = build_full_label(label, current_function_name);
                code_writer.write_if(&full_label);
            }
            Command::Function(function_name, n_vars) => {
                current_function_name = Some(function_name);
                code_writer.write_function(function_name, n_vars);
            }
            Command::Return => {
                code_writer.write_return();
            }
            Command::Call(function_name, n_args) => {
                code_writer.write_call(function_name, n_args);
            }
        }
    }
}
