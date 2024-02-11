# hack-vm-translator

Translates Hack VM code into Hack assembly.

## Features:
- Stack arithmetic
- Function call and return
- Branching
- Static variables

### Usage
Run with cargo:
```bash
cargo run -- <file_name>.vm
# or
cargo run -- <dir> # translates <dir>/*.vm to a single <dir>.asm
```
See the `test` directory for some sample .vm code.
### Examples
```bash
cargo run -- test/FunctionCalls/SimpleFunction/SimpleFunction.vm
```
#### SimpleFunction.vm
```
// Performs a simple calculation and returns the result.
// argument[0] and argument[1] must be set by the caller of this code.
function SimpleFunction.test 2
  push local 0
  push local 1
  add
  not
  push argument 0
  add
  push argument 1
  sub
  return
```
#### SimpleFunction.asm
```
(SimpleFunction.test)
D=0
@SP
A=M
M=D
@SP
M=M+1
D=0
@SP
A=M
M=D
@SP
M=M+1
// push local 0
@0
D=A
@LCL
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1
// push local 1
@1
D=A
@LCL
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1
// add
@SP
M=M-1
A=M
D=M
@R13
M=D
@SP
M=M-1
A=M
D=M
@R13
D=D+M
@SP
A=M
M=D
@SP
M=M+1
// not
@SP
M=M-1
A=M
D=M
D=!D
@SP
A=M
M=D
@SP
M=M+1
...
```

### Simulator
A CPU simulator for the Hack Computer is available [here](https://www.nand2tetris.org/software).

### Documentation
See Chapter 7-8 of [The Elements of Computing Systems: Building a Modern Computer from First Principles](https://www.amazon.com/Elements-Computing-Systems-Building-Principles/dp/0262640686) for the full specification of the Hack Virtual Machine.
