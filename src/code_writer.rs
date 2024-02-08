use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::path::Path;

use crate::parser::ArithmeticLogical;
use crate::parser::Command;

pub struct CodeWriter {
    file: File,
    file_name: String,
    logical_counter: usize, // guarantees unique label for logical op jumps
}

impl CodeWriter {
    pub fn build(file_name: &str) -> Result<CodeWriter, Error> {
        let path = Path::new(file_name);
        let file = File::create(file_name)?;
        Ok(CodeWriter {
            file,
            file_name: String::from(path.file_stem().unwrap().to_str().unwrap()),
            logical_counter: 0,
        })
    }

    pub fn write_arithmetic(&mut self, command: Command) {
        self.writeln(&format!("// {command}"));
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
        self.writeln(format!("// {command}").as_str());
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
                "temp" | "static" | "pointer" => self.writeln("A=A+D"),
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
            "temp" => format!("{}", 5 + index),
            _ => String::new(),
        }
    }

    fn writeln(&mut self, str: &str) {
        let _ = self.file.write_all(format!("{}\n", str).as_bytes());
    }
}
