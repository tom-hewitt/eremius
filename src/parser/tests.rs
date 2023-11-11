use crate::ir::Condition;
use crate::lexer::Lexer;
use crate::parser::{keywords::MNEMONICS, Line, Parser};
use std::fs;
use unicase::UniCase;

use crate::parser::keywords::Mnemonic;

extern crate test;
use test::{black_box, Bencher};

#[bench]
fn test_mnemonic_table(b: &mut Bencher) {
    b.iter(|| {
        let result = MNEMONICS.get(&UniCase::ascii("BLAL")).unwrap();
        assert_eq!(
            *result,
            Mnemonic::B {
                l: true,
                condition: Condition::AL
            }
        );

        black_box(result);
    })
}

#[bench]
fn test_repeated_instructions(b: &mut Bencher) {
    let instructions = "ADD R0, R1, R2\n".repeat(10561);

    fs::write("./repeated.a", &instructions).unwrap();

    b.iter(|| {
        let data = fs::read_to_string("./repeated.a").unwrap();

        let result = parse_to_vec(&data);

        black_box(result);
    });
}

fn parse_to_vec(input: &str) -> Vec<Line> {
    let parser = Parser {
        lexer: Lexer::new(input),
        line_count: 0,
    };

    parser
        .map(|x| match x {
            Ok(x) => x,
            Err(err) => panic!("{err}"),
        })
        .collect::<Vec<Line>>()
}

const BRANCH_EXAMPLES: &'static str = "B label ; branch unconditionally to label
BCC label ; branch to label if carry flag is clear
BEQ label ; branch to label if zero flag is set
MOV PC, #0 ; R15 = 0, branch to location zero
BL func ; subroutine call to function
func

MOV PC, LR ; R15=R14, return to instruction after the BL
MOV LR, PC ; store the address of the instruction
; after the next one into R14 ready to return
LDR PC, =func ; load a 32-bit value into the program counter";

#[test]
fn test_branch_examples() {
    insta::assert_debug_snapshot!(parse_to_vec(BRANCH_EXAMPLES));
}

const LOAD_STORE_EXAMPLES: &'static str = "LDR R1, [R0] ; Load R1 from the address in R0
LDR R8, [R3, #4] ; Load R8 from the address in R3 + 4
LDR R12, [R13, #-4] ; Load R12 from R13 - 4

STR R2, [R1, #0x100] ; Store R2 to the address in R1 + 0x100

LDRB R5, [R9] ; Load byte into R5 from R9
              ; (zero top 3 bytes) 
LDRB R3, [R8, #3] ; Load byte to R3 from R8 + 3
                  ; (zero top 3 bytes)
STRB R4, [R10, #0x200] ; Store byte from R4 to R10 + 0x200
LDR R11, [R1, R2] ; Load R11 from the address in R1 + R2
STRB R10, [R7, -R4] ; Store byte from R10 to addr in R7 - R4
LDR R11, [R3, R5, LSL #2] ; Load R11 from R3 + (R5 x 4)
LDR R1, [R0, #4]! ; Load R1 from R0 + 4, then R0 = R0 + 4
STRB R7, [R6, #-1]! ; Store byte from R7 to R6 - 1,
; then R6 = R6 - 1
LDR R3, [R9], #4 ; Load R3 from R9, then R9 = R9 + 4
STR R2, [R5], #8 ; Store R2 to R5, then R5 = R5 + 8

LDR R0, [PC, #40] ; Load R0 from PC + 0x40 (= address of
; the LDR instruction + 8 + 0x40)
LDR R0, [R1], R2 ; Load R0 from R1, then R1 = R1 + R2
";

#[bench]
fn bench_load_store_examples(b: &mut Bencher) {
    let input = LOAD_STORE_EXAMPLES;

    b.iter(|| {
        let parser = Parser {
            lexer: Lexer::new(&input),
            line_count: 0,
        };

        for line in parser {
            black_box(line).unwrap();
        }
    })
}

#[test]
fn test_load_store_examples() {
    // from the instruction set specification booklet
    insta::assert_debug_snapshot!(parse_to_vec(LOAD_STORE_EXAMPLES));
}

const LOAD_STORE_MULTIPLE_EXAMPLES: &'static str = "STMFD R13!, {R0 - R12, LR}
LDMFD R13!, {R0 - R12, PC}
LDMIA R0, {R5 - R8}
STMDA R1!, {R2, R5, R7 - R9, R11}";

#[test]
fn test_load_store_multiple_examples() {
    // from the instruction set specification booklet
    insta::assert_debug_snapshot!(parse_to_vec(LOAD_STORE_MULTIPLE_EXAMPLES));
}

const LAB1: &'static str = "	LDR	R0, tom
	LDR	R1, jill
	LDR	R2, jack
	LDR	R3, one
	LDR	R4, zero
loop	ADD	R0, R0, R1
	SUB	R2, R2, R3
	CMP	R2, R4
	BNE	loop
	SVC	2
jack	DEFW	3
jill	DEFW	4
tom	DEFW	0
one	DEFW	1
zero	DEFW	0";

#[test]
fn test_lab1() {
    // from the instruction set specification booklet
    insta::assert_debug_snapshot!(parse_to_vec(LAB1));
}

const LAB2_HELLO: &'static str = "; Hello Someone program - version 3

	B main

hello	DEFB	\"Hello \\0\"
goodbye	DEFB	\"and good-bye!\\n\\0\"
	ALIGN

main	ADR	R0, hello	; printf(\"Hello \")
	SVC 	3
	
start				; while R0 != 10 {// translate to ARM code
	
	SVC	1		; input a character to R0
	SVC	0		; output the character in R0
	
	CMP R0, #10
	BNE	start		; }// translate to ARM code

skip	ADR	R0, goodbye 	; printf(\"and good-bye!\")
	SVC	3

	SVC  	2		; stop the program
";

#[test]
fn test_lab2_hello() {
    // from the instruction set specification booklet
    insta::assert_debug_snapshot!(parse_to_vec(LAB2_HELLO));
}

const LAB2_AGE_HISTORY: &'static str = "; Age History

	B  main

born	DEFB 	\"you were born in \\0\"
were	DEFB 	\"you were \\0\"
in	DEFB 	\" in \\0\"
are	DEFB 	\"you are \\0\"
this	DEFB 	\" this year\\n\\0\"
	ALIGN

main
	LDR 	R4, =2022 	; present = 2022
	LDR 	R5, =2003 	; birth = 2003
	LDR 	R6, =0 	; year = 0
	LDR 	R7, =1 	; age = 1
	
	; this code does print \"you were born in \" + str(birth)
	ADR 	R0, born
	SVC 	3
	MOV 	R0, R5		; move birth into R0
	SVC 	4
	MOV 	R0, #10
	SVC 	0
	
	ADD 	R6, R5, #1 	; year = birth + 1
	
start	CMP 	R6, R4 	; while year != present {
	BEQ 	skip

	; this code does print \"you were \" + str(age) + \" in \" + str(year)
	ADR 	R0, were
	SVC 	3
	MOV 	R0, R7		; move age into R0
	SVC 	4
	ADR 	R0, in
	SVC 	3
	MOV 	R0, R6		; move year into R0
	SVC 	4
	MOV 	R0, #10
	SVC 	0

	ADD 	R6, R6, #1 	; year = year + 1
	
	ADD 	R7, R7, #1 	; age = age + 1
	
	B 	start 		; }

skip	; this code does print \"you are \" + str(age) + \"this year\"
	ADR 	R0, are
	SVC 	3
	MOV 	R0, R7 	; move age into R0
	SVC 	4
	ADR 	R0, this
	SVC 	3

	SVC 	2 		; stop
";

#[test]
fn test_lab2_age_history() {
    // from the instruction set specification booklet
    insta::assert_debug_snapshot!(parse_to_vec(LAB2_AGE_HISTORY));
}

const LAB3: &'static str = "	B part3 ; part1 or part2 or part3

buffer	DEFS 100,0

s1	DEFB \"one\\0\"
	ALIGN
s2	DEFB \"two\\0\"
	ALIGN
s3	DEFB \"three\\0\"
	ALIGN
s4	DEFB \"four\\0\"
	ALIGN
s5	DEFB \"five\\0\"
	ALIGN
s6	DEFB \"six\\0\"
	ALIGN
s7	DEFB \"seven\\0\"
	ALIGN
s8	DEFB \"twentytwo\\0\"
	ALIGN
s9	DEFB \"twenty\\0\"
	ALIGN

;************************** part 1 **************************
printstring
	LDRB 	R0, [R1], #1	; load the next character and increment R1
	CMP 	R0, #0		; check if it is the end of the string
	SVCNE 	0		; if its not the end, output the character
	BNE 	printstring	; if its not the end, loop
	MOV  	R0, #10	; given - output end-of-line
	SVC  	0		; given
	MOV  	PC, LR		; given

;************************** part 2 ***************************
strcat
	LDRB	R0, [R1], #1	; load the next character and increment R1
	CMP 	R0, #0		; check if it is the end of the string
	BNE	strcat		; if its not the end, loop
	SUB	R1, R1, #1	; subtract 1 to go back to the last character
cat	LDRB 	R0, [R2], #1	; load the next character and increment R2
	STRB	R0, [R1], #1	; store the character in the first string, and increment R1
	CMP	R0, #0		; check if its the end of the string
	BNE	cat		; if its not the end, loop
	MOV  	PC, LR		; given

strcpy
	LDRB 	R0, [R2], #1	; load the next character and increment R2
	STRB 	R0, [R1], #1	; store the character and increment R1
	CMP	R0, #0		; check if its the end of the string
	BNE 	strcpy		; if its not the end, loop
	MOV  	PC, LR		; given

;************************** part 3 **************************
sorted	STR 	LR, return2	; given
	LDRB	R4, [R2], #1	; get the next character from string 1 into R4, increment R2
	LDRB	R5, [R3], #1	; get the next character from string 2 into R5, increment R3
	CMP	R4, R5		; check if the character match
	BNE 	end		; if they don't, exit the loop
	CMP 	R4, #0		; check if the characters are zero
	BNE 	sorted		; if not, loop back to the start
end	CMP 	R4, R5		; compare the two characters
	LDR  	PC, return2 	; given
return2 DEFW 	0		; given

;*********************** the various parts ********************
part1	ADR R1, s1
	BL  printstring
	ADR R1, s2
	BL  printstring
	ADR R1, s3
	BL  printstring
	ADR R1, s4
	BL  printstring
	ADR R1, s5
	BL  printstring
	ADR R1, s6
	BL  printstring
	ADR R1, s7
	BL  printstring
	ADR R1, s8
	BL  printstring
	ADR R1, s9
	BL  printstring
	SVC 2

part2	ADR R2, s1
	ADR R1, buffer
	BL  strcpy
	ADR R1, buffer
	BL  printstring
	ADR R2, s2
	ADR R1, buffer
	BL  strcat
	ADR R1, buffer
	BL  printstring
	ADR R2, s3
	ADR R1, buffer
	BL  strcat
	ADR R1, buffer
	BL  printstring
	SVC 2

; used by part3
return4 DEFW 0,0
test2	STR LR, return4		; This mechanism will be improved later
	STR R3, return4+4	; Assembler will evaluate addition	
	MOV R0, R2
	SVC 3
	BL  sorted
	MOVLT R0, #'<'		; Three-way IF using conditions
	MOVEQ R0, #'='
	MOVGT R0, #'>'
	SVC 0
	LDR R0, return4+4
	SVC 3
	MOV R0, #10
	SVC 0
	LDR PC, return4

part3	ADR R2, s1
	ADR R3, s2
	BL  test2
	ADR R2, s2
	ADR R3, s3
	BL  test2
	ADR R2, s3
	ADR R3, s4
	BL  test2
	ADR R2, s4
	ADR R3, s5
	BL  test2
	ADR R2, s5
	ADR R3, s6
	BL  test2
	ADR R2, s6
	ADR R3, s7
	BL  test2
	ADR R2, s7
	ADR R3, s8
	BL  test2
	ADR R2, s8
	ADR R3, s9
	BL  test2
	ADR R2, s8
	ADR R3, s8
	BL  test2
	SVC 2
";

#[test]
fn test_lab3() {
    // from the instruction set specification booklet
    insta::assert_debug_snapshot!(parse_to_vec(LAB3));
}

const LAB4: &'static str = "; COMP15111 lab 4 - Template file

print_char	equ	0		; Define names to aid readability
stop		equ	2
print_str	equ	3
print_no	equ	4

cLF		equ	10		; Line-feed character


		ADR	SP, _stack	; set SP pointing to the end of our stack
		B	main

		DEFS	100		; this chunk of memory is for the stack
_stack					; This label is 'just after' the stack space


wasborn	DEFB	\"This person was born on \",0
was		DEFB	\"This person was \",0
on		DEFB	\" on \",0
is		DEFB	\"This person is \",0
today	DEFB	\" today!\",0
willbe	DEFB	\"This person will be \",0
		ALIGN

pDay	DEFW	23		;  pDay = 23    //or whatever is today's date
pMonth	DEFW	11		;  pMonth = 11  //or whatever is this month
pYear	DEFW	2005	;  pYear = 2005 //or whatever is this year

; def printDate (day, month, year)

; parameters
;  R0 = day
;  R1 = month
;  R2 = year

printDate	STMFD	SP!, {R0} ; callee saved registers
; print(str(day) + \"/\" + str(month) + \"/\" + str(year))
	SVC	print_no
	MOV	R0, #'/'
	SVC	print_char
	MOV	R0, R1
	SVC	print_no
	MOV	R0, #'/'
	SVC	print_char
	MOV	R0, R2
	SVC	print_no
	MOV	R0, #cLF
	SVC	print_char
	
; end of printDate
	LDMFD	SP!, {R0}	; callee saved registers
	MOV	PC, LR

; def printAgeHistory (bDay, bMonth, bYear)

; parameters
;  R0 = bDay (on entry, moved to R6 to allow SVC to output via R0)
;  R1 = bMonth
;  R2 = bYear
; local variables (callee-saved registers)
;  R4 = year
;  R5 = age
;  R6 = bDay - originally R0
;  R7 = pMonth

printAgeHistory	STMFD 	SP!, {R0, R4-R6}		; callee saved registers
		MOV 	R6, R0		; move to R6 to allow SVC to output

;   year = bYear + 1
		ADD	R4, R2, #1
;   age = 1;
		MOV	R5, #1

; print(\"This person was born on \" + printDate(bDay, bMonth, bYear))
		ADRL	R0, wasborn
		SVC	print_str
		MOV	R0, R6		; printDate day = bDay
		STMFD	SP!, {LR}	; calls another method so save LR
		BL	printDate
		LDMFD	SP!, {LR}	; retrieve saved LR

; this code does:
; while year < pYear or
;				(year == pYear and bMonth < pMonth) or
;				(year == pYear and bMonth == pMonth and bDay < pDay):
loop1		LDR	R0, pYear
		CMP	R4, R0		; compare year, pYear
		BLO	inner		; true if year < pYear (years are unsigned)
					; or
		BNE	or2		; (year == pYear and
		LDR	R7, pMonth
		CMP	R1, R7		; bMonth < pMonth)
		BLT	inner		; true if year == pYear and bMonth < pMonth
					; or
or2		CMP	R4, R0		; (year == pYear and
		BNE	end1		; false if year != pYear
		CMP 	R1, R7		; bMonth == pMonth and
		BNE 	end1		; false if bMonth != pMonth
		LDR	R0, pDay
		CMP	R6, R0		; bDay < pDay)
		BGE	end1		; false if bDay >= pDay
		
		


inner	;  print(\"This person was \" + str(age) + \" on \" + printDate(bDay, bMonth, year))
		ADRL	R0, was
		SVC	print_str
		MOV	R0, R5
		SVC	print_no
		ADRL	R0, on
		SVC	print_str
		MOV	R0, R6		; printDate day = bDay
		MOV 	R2, R4		; printDate year = year
		STMFD	SP!, {LR}	; calls another method so save LR
		BL	printDate
		LDMFD	SP!, {LR}	; retrieve saved LR

		; year = year + 1
		ADD	R4, R4, #1
		; age = age + 1
		ADD	R5, R5, #1
		; //}
		B	loop1

end1
; this code does:
; if (bMonth == pMonth and bDay == pDay):
		LDR	R0, pMonth
		CMP	R1, R0		; bMonth == pMonth
		BNE	else1
		LDR 	R0, pDay
		CMP 	R6, R0		; bDay = pDay
		BNE	else1

; print(\"This person is \" + str(age) + \" today!\")
		ADRL	R0, is
		SVC	print_str
		MOV	R0, R5
		SVC	print_no
		ADRL	R0, today
		SVC	print_str
		MOV	R0, #cLF
		SVC	print_char

; else
		B	end2
else1
; print(\"This person will be \" + str(age) + \" on \" + printDate(bDay, bMonth, year))
		ADRL	R0, willbe
		SVC	print_str
		MOV	R0, R5
		SVC	print_no
		ADRL	R0, on
		SVC	print_str
		MOV	R0, R6		; printDate day = bDay
		MOV 	R2, R4		; printDate year = year
		STMFD	SP!, {LR}	; calls another method so save LR
		BL	printDate
		LDMFD	SP!, {LR}	; retrieve saved LR

; }// end of printAgeHistory
end2	LDMFD	SP!, {R0, R4-R6}		; callee saved registers
		MOV	PC, LR

another	DEFB	\"Another person\",10,0
		ALIGN

; def main():
main
	LDR	R4, =&12345678		; Test value - not part of Java compilation
	MOV	R5, R4			; See later if these registers corrupted
	MOV	R6, R4

; printAgeHistory(pDay, pMonth, 2000)
		LDR	R0, pDay
		LDR R1, pMonth
		MOV R2, #2000
		BL printAgeHistory

; print(\"Another person\");
		ADRL	R0, another
		SVC	print_str

; printAgeHistory(13, 11, 2000)
		MOV	R0, #13
		MOV R1, #11
		MOV R2, #2000
		BL	printAgeHistory

	; Now check to see if register values intact (Not part of Java)
	LDR	R0, =&12345678		; Test value
	CMP	R4, R0			; Did you preserve these registers?
	CMPEQ	R5, R0			;
	CMPEQ	R6, R0			;

	ADRLNE	R0, whoops1		; Oh dear!
	SVCNE	print_str		;

	ADRL	R0, _stack		; Have you balanced pushes & pops?
	CMP	SP, R0			;

	ADRLNE	R0, whoops2		; Oh no!!
	SVCNE	print_str		; End of test code

; }// end of main
		SVC	stop


whoops1		DEFB	\"\\n** BUT YOU CORRUPTED REGISTERS!  **\\n\", 0
whoops2		DEFB	\"\\n** BUT YOUR STACK DIDN'T BALANCE!  **\\n\", 0
";

#[test]
fn test_lab4() {
    // from the instruction set specification booklet
    insta::assert_debug_snapshot!(parse_to_vec(LAB4));
}
