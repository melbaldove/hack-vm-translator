use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::path::PathBuf;

use crate::parser::ArithmeticLogical;
use crate::parser::Command;

pub struct CodeWriter {
    file: File,
    file_name: String,
    logical_counter: usize, // guarantees unique label for logical op jumps
    call_counter: usize,    // guarantees unique return labels
}

impl CodeWriter {
    pub fn build(path: PathBuf) -> Result<CodeWriter, Error> {
        let file = File::create(&path)?;
        let mut code_writer = CodeWriter {
            file,
            file_name: String::from(path.file_stem().and_then(|x| x.to_str()).unwrap()),
            logical_counter: 0,
            call_counter: 0,
        };

        code_writer.write_bootstrap();

        Ok(code_writer)
    }

    fn write_bootstrap(&mut self) {
        self.writeln("// bootstrap");
        self.writeln("@256");
        self.writeln("D=A");
        self.writeln("@SP");
        self.writeln("M=D");
        self.write_call("Sys.init", 0);
    }

    pub fn write_comment(&mut self, command: &Command) {
        self.writeln(&format!("// {command}"));
    }

    pub fn set_file_name(&mut self, file_name: String) {
        self.file_name = file_name
    }

    pub fn write_label(&mut self, label: &str) {
        self.writeln(&format!("({label})"));
    }

    pub fn write_goto(&mut self, label: &str) {
        self.writeln(&format!("@{label}"));
        self.writeln(&format!("0;JMP"));
    }

    pub fn write_if(&mut self, label: &str) {
        self.pop_to_d();
        self.writeln(&format!("@{label}"));
        self.writeln(&format!("D;JNE"));
    }

    pub fn write_arithmetic(&mut self, command: Command) {
        let command = match command {
            Command::ArithmeticLogical(arithmetic_logical) => arithmetic_logical,
            _ => return,
        };

        match command {
            ArithmeticLogical::Add => self.binary_op("+"),
            ArithmeticLogical::Sub => self.binary_op("-"),
            ArithmeticLogical::Neg => self.unary_op("-"),
            ArithmeticLogical::Eq => self.cmp("EQ"),
            ArithmeticLogical::Gt => self.cmp("GT"),
            ArithmeticLogical::Lt => self.cmp("LT"),
            ArithmeticLogical::And => self.binary_op("&"),
            ArithmeticLogical::Or => self.binary_op("|"),
            ArithmeticLogical::Not => self.unary_op("!"),
        }
    }

    pub fn write_push_pop(&mut self, command: Command) {
        match command {
            Command::Push(segment, index) => {
                self.set_a(segment, index);
                if segment == "constant" {
                    self.writeln("D=A");
                } else {
                    self.writeln("D=M"); // store segment[index]
                }
                self.push_d();
            }
            Command::Pop(segment, index) => {
                self.set_a(segment, index);
                self.writeln("D=A"); //  store address of segment[index]

                self.writeln("@R13");
                self.writeln("M=D"); // store &segment[index] to @R13

                self.pop_to_d();

                // store stack value to segment[index]
                self.writeln("@R13");
                self.writeln("A=M");
                self.writeln("M=D");
            } // no-op
            _ => {}
        };
    }

    pub fn write_function(&mut self, function_name: &str, n_vars: usize) {
        self.writeln(&format!("({function_name})"));
        // zeroes function's local segment before control transfers to it
        for _ in 0..n_vars {
            // todo: optimize and set 0 directly to M
            self.writeln(&format!("D=0"));
            self.push_d();
        }
    }

    pub fn write_call(&mut self, function_name: &str, n_args: usize) {
        let ret_label = format!("{function_name}$ret.{}", self.call_counter);
        self.call_counter += 1;
        // push return address
        self.writeln(&format!("@{ret_label}"));
        self.writeln("D=A");
        self.push_d();

        // push LCL
        self.writeln("@LCL");
        self.writeln("D=M");
        self.push_d();

        // push ARG
        self.writeln("@ARG");
        self.writeln("D=M");
        self.push_d();

        // push THIS
        self.writeln("@THIS");
        self.writeln("D=M");
        self.push_d();

        // push THAT
        self.writeln("@THAT");
        self.writeln("D=M");
        self.push_d();

        self.writeln("@SP");
        self.writeln("D=M");

        self.writeln("@LCL");
        self.writeln("M=D");

        // compute ARG = SP-5-n_args
        self.writeln("@5");
        self.writeln("D=D-A");
        self.writeln(&format!("@{n_args}"));
        self.writeln("D=D-A");
        self.writeln("@ARG");
        self.writeln("M=D");

        self.writeln(&format!("@{function_name}"));
        self.writeln(&format!("0;JMP"));
        self.writeln(&format!("({ret_label})"));
    }

    pub fn write_return(&mut self) {
        // frame = LCL
        self.writeln("@LCL");
        self.writeln("D=M");
        self.writeln("@R13");
        self.writeln("M=D");

        // retAddr = *(frame-5)
        self.writeln("@R13");
        self.writeln("D=M");
        self.writeln("@5");
        self.writeln("A=D-A");
        self.writeln("D=M");
        self.writeln("@R14");
        self.writeln("M=D");

        // *ARG = pop()
        self.pop_to_d();
        self.writeln("@ARG");
        self.writeln("A=M");
        self.writeln("M=D");

        // SP = ARG+1
        self.writeln("D=A+1");
        self.writeln("@SP");
        self.writeln("M=D");

        // THAT = *(frame-1)
        self.writeln("@R13");
        self.writeln("A=M-1");
        self.writeln("D=M");
        self.writeln("@THAT");
        self.writeln("M=D");

        // THIS = *(frame-2)
        self.writeln("@R13");
        self.writeln("D=M");
        self.writeln("@2");
        self.writeln("A=D-A");
        self.writeln("D=M");
        self.writeln("@THIS");
        self.writeln("M=D");

        // ARG = *(frame-3)
        self.writeln("@R13");
        self.writeln("D=M");
        self.writeln("@3");
        self.writeln("A=D-A");
        self.writeln("D=M");
        self.writeln("@ARG");
        self.writeln("M=D");

        // LCL = *(frame-4)
        self.writeln("@R13");
        self.writeln("D=M");
        self.writeln("@4");
        self.writeln("A=D-A");
        self.writeln("D=M");
        self.writeln("@LCL");
        self.writeln("M=D");

        // goto retAddr
        self.writeln("@R14");
        self.writeln("A=M");
        self.writeln("0;JMP");
    }

    // sets a to address of segment[index]
    fn set_a(&mut self, segment: &str, index: usize) {
        if segment == "constant" {
            self.writeln(&format!("@{index}"));
        } else {
            let addr = self.segment_to_addr(segment, index);
            // todo: optimize by only adding when index > 0
            self.writeln(&format!("@{index}"));
            self.writeln("D=A");
            self.writeln(&format!("@{addr}"));
            match segment {
                "temp" | "pointer" => self.writeln("A=A+D"),
                "static" => {}
                _ => self.writeln("A=M+D"),
            }
        }
    }

    fn push_d(&mut self) {
        self.writeln("@SP");
        self.writeln("A=M");
        self.writeln("M=D");
        self.increment_sp();
    }

    fn pop_to_d(&mut self) {
        self.decrement_sp();
        self.writeln("A=M");
        self.writeln("D=M");
    }

    fn increment_sp(&mut self) {
        self.writeln("@SP");
        self.writeln("M=M+1");
    }

    fn decrement_sp(&mut self) {
        self.writeln("@SP");
        self.writeln("M=M-1");
    }

    fn unary_op(&mut self, op: &str) {
        self.pop_to_d();
        self.writeln(&format!("D={op}D"));
        self.push_d();
    }

    fn binary_op(&mut self, op: &str) {
        self.pop_to_d();
        self.writeln("@R13");
        self.writeln("M=D");
        self.pop_to_d();
        self.writeln("@R13");
        self.writeln(&format!("D=D{op}M"));
        self.push_d();
    }

    fn cmp(&mut self, op: &str) {
        let cmp = &format!("CMP.{}", self.logical_counter);
        let end = &format!("END.{}", self.logical_counter);
        self.pop_to_d();
        self.writeln("@R13");
        self.writeln("M=D");
        self.pop_to_d();
        self.writeln("@R13");
        self.writeln("D=D-M");

        self.writeln(&format!("@{cmp}"));
        self.writeln(&format!("D;J{op}"));
        self.writeln("D=0");
        self.writeln(&format!("@{end}"));
        self.writeln("0;JMP");

        self.writeln(&format!("({cmp})"));
        self.writeln("D=-1");
        self.writeln(&format!("({end})"));
        self.push_d();
        self.logical_counter += 1;
    }

    fn segment_to_addr(&mut self, segment: &str, index: usize) -> String {
        match segment {
            "argument" => "ARG".to_owned(),
            "local" => "LCL".to_owned(),
            "static" => format!("{}.{}", self.file_name, index),
            "this" => "THIS".to_owned(),
            "that" => "THAT".to_owned(),
            "pointer" => "THIS".to_owned(),
            "temp" => format!("{}", 5),
            _ => String::new(),
        }
    }

    fn writeln(&mut self, str: &str) {
        let _ = self.file.write_all(format!("{}\n", str).as_bytes());
    }
}
