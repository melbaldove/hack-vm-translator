use std::{
    error::Error,
    fmt::{self, Display},
    str::Lines,
};

#[derive(Clone)]
pub struct Parser<'a> {
    file: Lines<'a>,
}

impl<'a> Parser<'a> {
    pub fn build(file_contents: &str) -> Result<Parser, Box<dyn Error>> {
        let file = file_contents.lines();
        Ok(Parser { file })
    }
}

#[derive(Debug)]
pub enum Command<'a> {
    ArithmeticLogical(ArithmeticLogical),
    Push(&'a str, usize),
    Pop(&'a str, usize),
    Label(&'a str),
    Goto(&'a str),
    If(&'a str),
    Function(&'a str),
    Return(),
    Call(&'a str),
}

impl<'a> Display for Command<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::ArithmeticLogical(al) => {
                write!(f, "{al}")
            }
            Command::Push(segment, index) => {
                write!(f, "push {segment} {index}")
            }
            Command::Pop(segment, index) => {
                write!(f, "pop {segment} {index}")
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

#[derive(Debug)]
pub enum ArithmeticLogical {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

impl Display for ArithmeticLogical {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ArithmeticLogical::Add => "add",
            ArithmeticLogical::Sub => "sub",
            ArithmeticLogical::Neg => "neg",
            ArithmeticLogical::Eq => "eq",
            ArithmeticLogical::Gt => "gt",
            ArithmeticLogical::Lt => "lt",
            ArithmeticLogical::And => "and",
            ArithmeticLogical::Or => "or",
            ArithmeticLogical::Not => "not",
        };
        write!(f, "{s}")
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Command<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = loop {
            match self.file.next() {
                None => return None,
                Some(line) => {
                    let line = line.trim();
                    let first_char = line.chars().nth(0).unwrap_or(' ');
                    if !line.is_empty() && first_char != '/' {
                        break Some(line);
                    }
                }
            }
        };
        let line = line.expect("Should have value");
        let command = parse_command(line).unwrap();
        Some(command)
    }
}

fn parse_command<'a>(line: &'a str) -> Result<Command<'a>, String> {
    if line.trim().is_empty() || line.starts_with('/') {
        return Err(format!("Error: Invalid line: {line}"));
    }
    let line = line.trim();
    // todo: explore one pass to optimize from O(2n) to O(n)
    let mut tokens = line.split_whitespace();

    let command = tokens
        .next()
        .ok_or_else(|| format!("Error: Invalid command: {line}"))?;

    let command = match command {
        "push" => {
            let segment = tokens
                .next()
                .ok_or_else(|| format!("Error: Expected segment for: {line}"))?;
            let segment = validate_segment(segment)?;
            let index = tokens
                .next()
                .ok_or_else(|| format!("Error: Expected index for: {line}"))?
                .parse::<usize>()
                .map_err(|_| format!("Error: Expected numeric index for: {line}"))?;
            Command::Push(segment, index)
        }
        "pop" => {
            let segment = tokens
                .next()
                .ok_or_else(|| format!("Error: Expected segment for: {line}"))?;
            let segment = validate_segment(segment)?;
            let index = tokens
                .next()
                .ok_or_else(|| format!("Error: Expected index for: {line}"))?
                .parse::<usize>()
                .map_err(|_| format!("Error: Expected numeric index for: {line}"))?;

            Command::Pop(segment, index)
        }
        "add" => Command::ArithmeticLogical(ArithmeticLogical::Add),
        "sub" => Command::ArithmeticLogical(ArithmeticLogical::Sub),
        "neg" => Command::ArithmeticLogical(ArithmeticLogical::Neg),
        "eq" => Command::ArithmeticLogical(ArithmeticLogical::Eq),
        "gt" => Command::ArithmeticLogical(ArithmeticLogical::Gt),
        "lt" => Command::ArithmeticLogical(ArithmeticLogical::Lt),
        "and" => Command::ArithmeticLogical(ArithmeticLogical::And),
        "or" => Command::ArithmeticLogical(ArithmeticLogical::Or),
        "not" => Command::ArithmeticLogical(ArithmeticLogical::Not),
        _ => return Err(format!("Error: Invalid command: {line}")),
    };
    Ok(command)
}

fn validate_segment<'a>(segment: &'a str) -> Result<&'a str, String> {
    match segment {
        "argument" | "constant" | "local" | "static" | "this" | "that" | "pointer" | "temp" => {
            Ok(segment)
        }
        _ => Err(format!("Error: Invalid segment: {segment}")),
    }
}
#[cfg(test)]
mod tests {
    use super::{parse_command, Command};

    #[test]
    fn parse_push_command() {
        match parse_command("push local 3").unwrap() {
            Command::Push(segment, index) => {
                assert_eq!(segment, "local");
                assert_eq!(index, 3);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_pop_command() {
        match parse_command("pop this 4").unwrap() {
            Command::Push(segment, index) => {
                assert_eq!(segment, "this");
                assert_eq!(index, 4);
            }
            _ => panic!(),
        }
    }
}
