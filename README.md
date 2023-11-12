# Eremius

an assembler, emulator, and debugger for a subset of the ARM assembly language

### 🚧 This project is a work in progress 🚧

- ✅ The assembler is (mostly!) complete for the chosen subset of the specification.
- 🚧 The emulator is in the middle of a rewrite to align its functionality with the exact behaviour specified in the ARM manual, so isn't currently fully functional.
- 🚧 There is an accompanying browser-based debugger that is currently in a different repository - I'm working on polishing it up and then I'm planning to integrate the entire project into a single monorepo.

Feel free to get in touch or raise a github issue, and check back soon for updates!

## Contents

1. [Supported Mnemonics](#supported-mnemonics)
2. [Condition Flags](#condition-flags)
3. [Operands](#operands)
4. [Labels](#labels)
5. [Instructions](#instructions)
6. [Assembler Overview](#assembler-overview)
7. [Testing](#testing)

## Supported Mnemonics

| Category            | Mnemonic                               | Status |
| ------------------- | -------------------------------------- | ------ |
| Branch              | [B](#b---branch)                       | ✅     |
| Data Processing     | [ADD](#add---add)                      | ✅     |
|                     | [SUB](#sub---subtract)                 | ✅     |
|                     | [CMP](#cmp---compare)                  | ✅     |
|                     | [MOV](#mov---move)                     | ✅     |
| Data Transfer       | [LDR](#ldr---load-register)            | ✅     |
|                     | [STR](#str---store-register)           | ✅     |
|                     | [LDRB](#ldrb---load-register-byte)     | ✅     |
|                     | [STRB](#strb---store-register-byte)    | ✅     |
|                     | [LDM](#ldm---load-multiple)            | ✅     |
|                     | [STM](#stm---store-multiple)           | ✅     |
| System Call         | [SVC](#svc---supervisor-call)          | ✅     |
| Pseudo-Instruction  | [ADR](#adr---address-register)         | ✅     |
| Assembler Directive | [DEFW](#defw---define-words)           | ✅     |
|                     | [DEFB](#defb---define-byte)            | ✅     |
|                     | [DEFS](#defs---define-space)           | ✅     |
|                     | [ORIGIN](#origin---set-origin-address) | 🚧     |
|                     | [ALIGN](#align---align-address)        | ✅     |
|                     | [ENTRY](#entry---set-entry-point)      | 🚧     |
|                     | [EQU](#equ---equals)                   | ✅     |

## Condition Flags
| Mnemonic Extension | Meaning   |
| ------------------ | --------- |
| `EQ`               | Equal     |
| `NE`               | Not Equal |
| `CS`/`HS`          | Carry Set/Unsigned Higher or Same |
| `CC`/`LO`          | Carry Clear/Unsigned Lower |
| `MI`               | Minus (Negative) |
| `PL`               | Plus (Positive or Zero) |
| `VS`               | Overflow |
| `VC`               | No Overflow |
| `HI`               | Unsigned Higher |
| `LS`               | Unsigned Lower or Same |
| `GE`               | Signed Greater Than or Equal |
| `LT`               | Signed Less Than |
| `GT`               | Signed Greater Than |
| `LE`               | Signed Less Than or Equal |
| `AL`               | Always (Unconditional) |

## Operands
### Shifter Operands
There are 4 types of shifter operands:

| Format                        | Name                          |
| ----------------------------- | ----------------------------- |
| `#<immediate>`                | Immediate                     |
| `<Rm>`                        | Register                      |
| `<Rm>, <shift> #<shift_imm>`  | Register Shift By Immediate   |
| `<Rm>, <shift> <Rs>`          | Register Shift By Register    |

### Load/Store Address Operands
All addressing modes involve a base register and an offset.

There are 3 types of offset value:

| Format             | Name   |
| ------------------ | --------- |
| `#+/-<offset_12>`                | Immediate            |
| `+/-<Rm>`                        | Register             |
| `+/-<Rm>, <shift> #<shift_imm>`  | Scaled Register      |

There are also 3 types of offset:

| Format               | Name         |
| -------------------- | ------------ |
| `[<Rn>, #<offset>]`  | Offset       |
| `[<Rn>, #<offset>]!` | Pre-Indexed  |
| `[<Rn>], #<offset>`  | Post-Indexed |

The 9 combinations of these formats form the 9 possible addressing modes.

## Labels
A Label is a program-relative address that can be assigned to any line in the program.

## Instructions
### B - Branch
Causes a branch to a target address.
#### Syntax
```
B{L}{<cond>} <target_address>
```

#### Flags
|        | Behaviour |
| ------ | --------- |
|`L`     | Specifies that the instruction should store a return address in the link register (R14) |
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|

#### Operands
|                   | Behaviour |
| ----------------- |-----------|
|`<target_address>` | Specifies the address to branch to |

### ADD - Add
Adds two values. Can optionally update the condition flags based on the result.
#### Syntax
```
ADD{<cond>}{S} <Rd>, <Rn>, <shifter_operand>
```

#### Flags
|        | Behaviour |
|--------|-----------|
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|
|`S`     | Specifies that the instruction should update the Current Program Status Register (CPSR) Flags |

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rd>`               | Specifies the destination register |
|`<Rn>`               | Specifies the register that contains the first operand |
|`<shifter_operand>`  | Specifies the second operand (see [Shifter Operands](#shifter-operands))

### SUB - Subtract
Subtracts one value from another. Can optionally update the condition flags based on the result.
#### Syntax
```
ADD{<cond>}{S} <Rd>, <Rn>, <shifter_operand>
```

#### Flags
|        | Behaviour |
|--------|-----------|
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|
|`S`     | Specifies that the instruction should update the Current Program Status Register (CPSR) Flags |

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rd>`               | Specifies the destination register |
|`<Rn>`               | Specifies the register that contains the first operand |
|`<shifter_operand>`  | Specifies the second operand (see [Shifter Operands](#shifter-operands))

### CMP - Compare
Compares two values, always updating the condition flags.
#### Syntax
```
CMP{<cond>} <Rn>, <shifter_operand>
```

#### Flags
|        | Behaviour |
|--------|-----------|
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rn>`               | Specifies the register that contains the first operand |
|`<shifter_operand>`  | Specifies the second operand (see [Shifter Operands](#shifter-operands))

### MOV - Move
Writes a value to a register.
#### Syntax
```
MOV{<cond>}{S} <Rd>, <shifter_operand>
```

#### Flags
|        | Behaviour |
|--------|-----------|
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|
|`S`     | Specifies that the instruction should update the Current Program Status Register (CPSR) Flags |

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rd>`               | Specifies the destination register |
|`<shifter_operand>`  | Specifies the operand (see [Shifter Operands](#shifter-operands))

#### Flags
|        | Behaviour |
|--------|-----------|
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|
|`S`     | Specifies that the instruction should update the Current Program Status Register (CPSR) Flags |

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rd>`               | Specifies the destination register |
|`<shifter_operand>`  | Specifies the operand (see [Shifter Operands](#shifter-operands))

### LDR - Load Register
Loads a word into a register.

When used with a constant, this is a psuedo-instruction that the assembler will replace with either a data processing isntruction or an `LDR` instruction pointing to a literal in memory.

#### Syntax
```
LDR{<cond>} <Rd>, <source>
```

#### Flags
|        | Behaviour |
|--------|-----------|
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rd>`               | Specifies the destination register |
|`<source>`  | Specifies the address (see [Load/Store Address Operands](#loadstore-address-operands)) or a constant expression prefixed with `=` |

#### Examples
```
LDR r1, =0xfff; pseudo-instruction to load the constant 0xfff into r1
```

### STR - Store Register
Stores a word to memory.
#### Syntax
```
STR{<cond>} <Rd>, <address>
```

#### Flags
|        | Behaviour |
|--------|-----------|
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rd>`               | Specifies the source register |
|`<address>`  | Specifies the address (see [Load/Store Address Operands](#loadstore-address-operands)) |

### LDRB - Load Register Byte
Loads a byte from memory into a register and zero-entends it to a word.

#### Syntax
```
LDRB{<cond>} <Rd>, <address>
```

#### Flags
|        | Behaviour |
|--------|-----------|
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rd>`               | Specifies the destination register |
|`<address>`  | Specifies the address (see [Load/Store Address Operands](#loadstore-address-operands)) |

### STRB - Store Register Byte
Stores the least significant byte of a register to memory.
#### Syntax
```
STRB{<cond>} <Rd>, <address>
```

#### Flags
|        | Behaviour |
|--------|-----------|
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rd>`               | Specifies the source register |
|`<address>`  | Specifies the address (see [Load/Store Address Operands](#loadstore-address-operands)) |

### LDM - Load Multiple
Loads values into multiple registers from sequential memory locations.
#### Syntax
```
LDM{<cond>}<addressing_mode> <Rn>{!}, <registers>
```

#### Flags
|                        | Behaviour |
| ---------------------- |-----------|
|`<cond>`                | Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|
|`<addressing_mode>`     | Specifies how to produce a sequential range of addresses (see [Load Multiple Addressing Modes](#load-multiple-addressing-modes)) |

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rn>`               | Specifies the base register used by `<addressing_mode>`, which can be optionally written back to if followed by `!` |
|`<registers>`  | Specifies the list of registers to be loaded, separated by commas and surrounded by `{` and `}` |

#### Load Multiple Addressing Modes
|          | Name            |
| -------- | --------------- |
|`IB`/`ED` | Increment Before/Empty Descending Stack |
|`IA`/`FD` | Increment After/Full Descending Stack   |
|`DB`/`EA` | Decrement Before/Empty Ascending Stack  |
|`DA`/`FA` | Decrement After/Full Ascending Stack    |

### STM - Store Multiple
Stores values from multiple registers into sequential memory locations.
#### Syntax
```
STM{<cond>}<addressing_mode> <Rn>{!}, <registers>
```

#### Flags
|                        | Behaviour |
| ---------------------- |-----------|
|`<cond>`                | Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|
|`<addressing_mode>`     | Specifies how to produce a sequential range of addresses (see [Load Multiple Addressing Modes](#load-multiple-addressing-modes)) |

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<Rn>`               | Specifies the base register used by `<addressing_mode>`, which can be optionally written back to if followed by `!` |
|`<registers>`  | Specifies the list of registers to be stored, separated by commas and surrounded by `{` and `}` |

#### Store Multiple Addressing Modes
|          | Name            |
| -------- | --------------- |
|`IB`/`FA` | Increment Before/Full Ascending Stack   |
|`IA`/`EA` | Increment After/Empty Ascending Stack   |
|`DB`/`FD` | Decrement Before/Full Descending Stack  |
|`DA`/`ED` | Decrement After/Empty Descending Stack  |

### SVC - SuperVisor Call
Calls a system function.
#### Syntax
```
SVC{<cond>} <immed_24>
```

#### Flags
|        | Behaviour |
|--------|-----------|
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|

#### Operands
|                     | Behaviour |
| ------------------- | --------- |
|`<immed24>`     | Specifies what system function is being requested (see [System Functions](#system-functions)) |

#### System Functions
|        | Behaviour |
|--------|-----------|
|`0` | Outputs the character in `R0` |
|`1` | Inputs a character into `R0` |
|`2` | Halts the program |
|`3` | Outputs the C string starting at the address in `R0` |
|`4` | Outputs the number in `R0` as a decimal |

### ADR - Address Register
Loads an address into a register.

This is a psuedo-instruction that will be replaced with either one or two data processing instructions. Not all addresses can be generated into one instruction, so the `L` flag exists to allow two instructions to be generated.
#### Syntax
```
ADR{<cond>}{L} <Rd>, <target_address>
```

#### Flags
|        | Behaviour |
| ------ | --------- |
|`<cond>`| Specifies under what circumstances the instruction should be executed (see [Condition Flags](#condition-flags))|
|`L`     | Specifies whether to allow assembling this pseudo-instruction into two data processing instructions rather than one, allowing for a wider range of addresses |

#### Operands
|                   | Behaviour   |
| ----------------- | ----------- |
|`<Rd>`             | Specifies the destination register |
|`<target_address>` | Specifies the address to load |

### DEFB - Define Bytes
Reserves one or multiple bytes of space in memory and puts initial values in them.

This is an assembler directive, and will not generate any actual instructions.
#### Syntax
```
DEFB <expression>{, ...}
```

#### Operands
|         | Behaviour   |
| ------- | ----------- |
|`<expression>` | Specifies the value to put in the word |

#### Examples
```
string DEFB "Hello", 0
```

### DEFW - Define Words
Reserves one or multiple words of space in memory and puts initial values in them.

This is an assembler directive, and will not generate any actual instructions.
#### Syntax
```
DEFW <expression>{, ...}
```

#### Operands
|         | Behaviour   |
| ------- | ----------- |
|`<expression>` | Specifies the value to put in the word |

#### Examples
```
square table DEFW 0, 1, 4, 9, 16, 25
```

### DEFB - Define Byte
Reserves a byte of space in memory and puts an initial value in it.

This is an assembler directive, and will not generate any actual instructions.
#### Syntax
```
DEFW <expression>
```

#### Operands
|         | Behaviour   |
| ------- | ----------- |
|`<expression>` | Specifies the value to put in the word |

### DEFS - Define Space
Reserves a block of space in memory.

This is an assembler directive, and will not generate any actual instructions.
#### Syntax
```
DEFS <size> {, <fill>}
```

#### Operands
|         | Behaviour   |
| ------- | ----------- |
|`<size>` | Specifies size of the block to reserve |
|`<fill>` | Specifies an optional value to fill each byte in the space with |

### ORIGIN - Set Origin Address
Sets the address of the following code.

This is an assembler directive, and will not generate any actual instructions.
#### Syntax
```
ORIGIN <target_address>
```

#### Operands
|                   | Behaviour   |
| ----------------- | ----------- |
|`<target_address>` | Specifies the address to place the following code |

### ALIGN - Align Address
Aligns the following code to the next word boundary.

This is an assembler directive, and will not generate any actual instructions.
#### Syntax
```
ALIGN
```

### ENTRY - Set Entry Point
Places the following code at the start of the program, serving as the entry point.

This is an assembler directive, and will not generate any actual instructions.
#### Syntax
```
ENTRY
```

### EQU - Equals
Defines a name for a literal value.

This is an assembler directive, and will not generate any actual instructions.
#### Syntax
```
discount EQU 100
...
SUB R5, R2, #discount
```

## Assembler Overview
The Assembler is broken down into multiple stages and uses multiple intermediate representations. I've found this makes the code more modular and easier to reason about. These are mostly zero-cost abstractions as they make heavy use of Rust Iterators. There is only one point where we have to take into account the entire program, which is the symbol resolution step. This is the only intermediate step where we make a complete pass of the program - it can still be considered a two-pass process, like most assemblers.

### Step 1 - Lexer
Converts a string to tokens.

### Step 2 - Parser
Converts tokens to an AST (Abstract Syntax Tree), consisting of Statements. A Statement can contain a label and a comment, both optional.

The AST is a one-to-one structured representation of what the user wrote, making it easier to work with. No other alterations are made at this stage.

Example:
```
start	CMP 	R6, R4
```
gets converted to
```rs
Line {
    label: Some(
        "start",
    ),
    statement: Some(
        Instruction {
            kind: DataProcessing {
                condition: AL,
                kind: Comparison {
                    kind: CMP,
                    source: Register(
                        6,
                    ),
                    shifter: Register(
                        Register(
                            4,
                        ),
                    ),
                },
            },
        },
    ),
}
```

### Step 3 - Builder
Builds the symbol table and literal pools.


Converts the AST to a High-Level Intermediate Representation (HIR) and constructs a symbol table. The Statements are converted to Instructions (with the Operands left unresolved).

In this stage, comments and empty lines are discarded, directives are applied, and psuedo-instructions are expanded. This lets us decide the final memory addresses of each instruction and piece of data.

### Step 4 - Symbol Resolver
Converts the High-Level Intermediate Representation (HIR) to a Low-Level Intermediate Representation (LIR) by resolving symbols and encoding immediates.

The LIR is a one-to-one structured representation of the machine code. This is also the format used by the emulator.

### Step 5 - Encoder
Converts Instructions and Data into 32-bit words.

## Testing
There are some snapshot tests to check for regressions. These can be run using the `cargo test` command.
