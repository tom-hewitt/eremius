#![feature(iter_intersperse)]

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use unicase::UniCase;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let mut builder = phf_codegen::Map::new();

    mnemonics().build(&mut builder);

    writeln!(
        &mut file,
        "pub static MNEMONICS: phf::Map<UniCase<&'static str>, Mnemonic> = \n{};\n",
        builder.build()
    )
    .unwrap();
}

type Builder = phf_codegen::Map<UniCase<String>>;

const CONDITION_FLAG: Flag<18> = Flag::union(
    "condition",
    [
        ("", "Condition::AL"),
        ("AL", "Condition::AL"),
        ("EQ", "Condition::EQ"),
        ("NE", "Condition::NE"),
        ("CS", "Condition::CS"),
        ("HS", "Condition::CS"),
        ("CC", "Condition::CC"),
        ("LO", "Condition::CC"),
        ("MI", "Condition::MI"),
        ("PL", "Condition::PL"),
        ("VS", "Condition::VS"),
        ("VC", "Condition::VC"),
        ("HI", "Condition::HI"),
        ("LS", "Condition::LS"),
        ("GE", "Condition::GE"),
        ("LT", "Condition::LT"),
        ("GT", "Condition::GT"),
        ("LE", "Condition::LE"),
    ],
);

const LINK_FLAG: Flag<2> = Flag::bool("l");

const SET_FLAGS_FLAG: Flag<2> = Flag::bool("s");

const LDM_MODE_FLAG: Flag<8> = Flag::union(
    "mode",
    [
        // IA
        ("IA", "MultipleAddressingMode::IncrementAfter"),
        // Full Descending Stack
        ("FD", "MultipleAddressingMode::IncrementAfter"),
        // IB
        ("IB", "MultipleAddressingMode::IncrementBefore"),
        // Empty Ascending Stack
        ("ED", "MultipleAddressingMode::IncrementBefore"),
        // DA
        ("DA", "MultipleAddressingMode::DecrementAfter"),
        // Full Ascending Stack
        ("FA", "MultipleAddressingMode::DecrementAfter"),
        // DB
        ("DB", "MultipleAddressingMode::DecrementBefore"),
        // Empty Ascending Stack
        ("EA", "MultipleAddressingMode::DecrementBefore"),
    ],
);

const STM_MODE_FLAG: Flag<8> = Flag::union(
    "mode",
    [
        // IA
        ("IA", "MultipleAddressingMode::IncrementAfter"),
        // Empty Ascending Stack
        ("EA", "MultipleAddressingMode::IncrementAfter"),
        // IB
        ("IB", "MultipleAddressingMode::IncrementBefore"),
        // Full Ascending Stack
        ("FA", "MultipleAddressingMode::IncrementBefore"),
        // DA
        ("DA", "MultipleAddressingMode::DecrementAfter"),
        // Empty Descending Stack
        ("ED", "MultipleAddressingMode::DecrementAfter"),
        // DB
        ("DB", "MultipleAddressingMode::DecrementBefore"),
        // Full Descending Stack
        ("FD", "MultipleAddressingMode::DecrementBefore"),
    ],
);

const LONG_FLAG: Flag<2> = Flag::bool("l");

fn mnemonics() -> impl MnemonicTableBuilder {
    MnemonicTable
        // branch
        .entry("B".then(LINK_FLAG).then(CONDITION_FLAG))
        // data processing
        .entry("ADD".then(CONDITION_FLAG).then(SET_FLAGS_FLAG))
        .entry("SUB".then(CONDITION_FLAG).then(SET_FLAGS_FLAG))
        .entry("CMP".then(CONDITION_FLAG))
        .entry("MOV".then(CONDITION_FLAG).then(SET_FLAGS_FLAG))
        // load/store
        .entry("LDR".then(CONDITION_FLAG))
        .entry("STR".then(CONDITION_FLAG))
        .entry("LDRB".then(CONDITION_FLAG))
        .entry("STRB".then(CONDITION_FLAG))
        .entry("LDM".then(CONDITION_FLAG).then(LDM_MODE_FLAG))
        .entry("STM".then(CONDITION_FLAG).then(STM_MODE_FLAG))
        // supervisor call
        .entry("SVC".then(CONDITION_FLAG))
        // pseudo-instructions
        .entry("ADR".then(LONG_FLAG).then(CONDITION_FLAG))
        // directives
        .entry("DEFS")
        .entry("DEFB")
        .entry("DEFW")
        .entry("ALIGN")
        .entry("ORIGIN")
        .entry("ENTRY")
        .entry("EQU")
}

trait MnemonicTableBuilder: Sized {
    fn build(self, builder: &mut Builder);

    fn entry<M: MnemonicBuilder>(self, mnemonic: M) -> MnemonicEntry<Self, M> {
        MnemonicEntry {
            table: self,
            mnemonic,
        }
    }
}

struct MnemonicTable;

impl MnemonicTableBuilder for MnemonicTable {
    fn build(self, _builder: &mut Builder) {}
}

struct MnemonicEntry<T: MnemonicTableBuilder, M: MnemonicBuilder> {
    table: T,
    mnemonic: M,
}

impl<T: MnemonicTableBuilder, M: MnemonicBuilder> MnemonicTableBuilder for MnemonicEntry<T, M> {
    fn build(self, builder: &mut Builder) {
        self.table.build(builder);

        self.mnemonic.build(builder, "".to_owned(), &vec![]);
    }
}

trait MnemonicBuilder: Sized {
    fn build(
        &self,
        builder: &mut Builder,
        flags: String,
        fields: &Vec<(&'static str, &'static str)>,
    );

    fn then<const C: usize>(self, flag: Flag<C>) -> MnemonicWithFlag<Self, C> {
        MnemonicWithFlag {
            mnemonic: self,
            flag,
        }
    }
}

impl MnemonicBuilder for &'static str {
    fn build(
        &self,
        builder: &mut Builder,
        flags: String,
        fields: &Vec<(&'static str, &'static str)>,
    ) {
        let fields_string = fields
            .iter()
            .map(|(flag, value)| format!("{}: {}", flag.to_lowercase(), value))
            .reduce(|prev, field| format!("{prev}, {field}"));

        let field_struct_string = match fields_string {
            None => "".to_owned(),
            Some(fields) => format!(" {{ {fields} }}"),
        };

        builder.entry(
            UniCase::new(format!("{self}{flags}")),
            &format!("Mnemonic::{self}{field_struct_string}"),
        );
    }
}

struct MnemonicWithFlag<M: MnemonicBuilder, const C: usize> {
    mnemonic: M,
    flag: Flag<C>,
}

impl<M: MnemonicBuilder, const C: usize> MnemonicBuilder for MnemonicWithFlag<M, C> {
    fn build(
        &self,
        builder: &mut Builder,
        flags: String,
        fields: &Vec<(&'static str, &'static str)>,
    ) {
        for (flag, value) in self.flag.members {
            self.mnemonic.build(
                builder,
                format!("{flag}{flags}"),
                // add this flag to the start of the fields vec
                &[&[(self.flag.name, value)], fields.as_slice()].concat(),
            )
        }
    }
}

struct Flag<const COMBINATIONS: usize> {
    name: &'static str,
    members: [(&'static str, &'static str); COMBINATIONS],
}

impl<const C: usize> Flag<C> {
    const fn union(name: &'static str, members: [(&'static str, &'static str); C]) -> Self {
        Flag { name, members }
    }
}

impl Flag<2> {
    const fn bool(name: &'static str) -> Flag<2> {
        Flag {
            name,
            members: [(name, "true"), ("", "false")],
        }
    }
}
