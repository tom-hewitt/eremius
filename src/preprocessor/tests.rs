use crate::ir::{CalculationKind, Condition, DataProcessingKind, InstructionKind, Rd, Rn};
use crate::parser::{
    DiadicOperator, DirectiveKind, Expression, Line, PseudoInstructionKind, Register, SetFlags,
    ShifterOperandExpression, Statement, Symbol,
};

use crate::preprocessor::PreProcessor;

#[test]
fn test_label_adrl_example() {
    let lines = [
        Ok(Line {
            label: Some("func".to_owned()),
            statement: None,
        }),
        Ok(Line {
            label: None,
            statement: Some(Statement::PseudoInstruction {
                kind: PseudoInstructionKind::LoadRegisterConstant {
                    condition: Condition::AL,
                    destination: Rd(15),
                    value: Expression::Symbol(Symbol("func".to_owned())),
                },
            }),
        }),
    ];

    let result = PreProcessor::new().run(lines.into_iter());

    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_origin() {
    let lines = [
        Ok(Line {
            label: Some("func".to_owned()),
            statement: Some(Statement::Instruction {
                kind: InstructionKind::DataProcessing {
                    condition: Condition::AL,
                    kind: DataProcessingKind::Calculation {
                        kind: CalculationKind::ADD,
                        set_flags: SetFlags::DontSet,
                        destination: Rd(0),
                        source: Rn(1),
                        shifter: ShifterOperandExpression::Register(Register(2)),
                    },
                },
            }),
        }),
        Ok(Line {
            label: None,
            statement: Some(Statement::Directive {
                kind: DirectiveKind::Origin {
                    address: Expression::Diadic(
                        Box::new(Expression::Symbol(Symbol("func".to_owned()))),
                        DiadicOperator::Plus,
                        Box::new(Expression::Number { base: 10, n: 12 }),
                    ),
                },
            }),
        }),
        Ok(Line {
            label: Some("func".to_owned()),
            statement: Some(Statement::Instruction {
                kind: InstructionKind::DataProcessing {
                    condition: Condition::AL,
                    kind: DataProcessingKind::Calculation {
                        kind: CalculationKind::ADD,
                        set_flags: SetFlags::DontSet,
                        destination: Rd(0),
                        source: Rn(1),
                        shifter: ShifterOperandExpression::Register(Register(2)),
                    },
                },
            }),
        }),
    ];

    let result = PreProcessor::new().run(lines.into_iter());

    insta::assert_debug_snapshot!(result);
}
