mod tests;

#[derive(Debug, PartialEq)]
pub enum InstructionKind<
    BranchAddress = SignedImm<24>,
    LoadStoreAddress = LoadStoreAddressCode<Imm<12>, Imm<5>>,
    ShifterOperand = ShifterOperandCode<RotatedImm8, Imm<5>>,
    SuperVisorCallNumber = Imm<24>,
> {
    Branch {
        condition: Condition,
        kind: BranchKind,
        target: BranchAddress,
    },

    DataProcessing {
        condition: Condition,
        kind: DataProcessingKind<ShifterOperand>,
    },

    LoadStore {
        condition: Condition,
        kind: LoadStoreKind,
        quantity: LoadStoreQuantity,
        destination: Rd,
        address: LoadStoreAddress,
    },
    LoadStoreMultiple {
        condition: Condition,
        kind: LoadStoreKind,
        mode: MultipleAddressingMode,
        base: Rn,
        write_back: WriteBack,
        register_list: RegisterList,
    },

    SuperVisorCall {
        condition: Condition,
        immediate: SuperVisorCallNumber,
    },
}

#[derive(Debug, PartialEq)]
pub enum DataProcessingKind<ShifterOperand = ShifterOperandCode> {
    Move {
        kind: MoveKind,
        set_flags: SetFlags,
        destination: Rd,
        shifter: ShifterOperand,
    },
    Comparison {
        kind: ComparisonKind,
        source: Rn,
        shifter: ShifterOperand,
    },
    Calculation {
        kind: CalculationKind,
        set_flags: SetFlags,
        destination: Rd,
        source: Rn,
        shifter: ShifterOperand,
    },
}

#[derive(Debug, PartialEq)]
pub enum ComparisonKind {
    CMP,
    // CMN,
    // TST,
    // TEQ,
}

#[derive(Debug, PartialEq)]
pub enum CalculationKind {
    ADD,
    SUB,
    // RSB,
    // ADC,
    // SBC,
    // RSC,
    // AND,
    // BIC,
    // EOR,
    // ORR,
}

#[derive(Debug, PartialEq)]
pub enum LoadStoreKind {
    Load,
    Store,
}

#[derive(Debug, PartialEq)]
pub struct LoadStoreAddressCode<Immediate, ShiftImmediate> {
    pub base: Rn,
    pub offset: AddressingOffset<Immediate, ShiftImmediate>,
}

#[derive(Debug, PartialEq)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, PartialEq)]
pub struct AddressingOffset<Immediate, ShiftImmediate> {
    pub sign: Sign,
    pub value: AddressingOffsetValue<Immediate, ShiftImmediate>,
    pub mode: OffsetMode,
}

#[derive(Debug, PartialEq)]
pub enum AddressingOffsetValue<Immediate, ShiftImmediate> {
    Immediate(Immediate),
    Register(Rm),
    ScaledRegister(ShiftedRegister<ShiftImmediate>),
}

#[derive(Debug, PartialEq)]
pub enum OffsetMode {
    Offset,
    PreIndexed,
    PostIndexed,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shift {
    LogicalShiftLeft = 0b00,
    LogicalShiftRight = 0b01,
    ArithmeticShiftRight = 0b10,
    RotateRight = 0b11,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    /// equal
    EQ = 0b0000,

    /// not equal
    NE = 0b0001,

    /// unsigned higher or same (alias: HS)
    CS = 0b0010,

    /// unsigned lower (alias: LO)
    CC = 0b0011,

    /// negative
    MI = 0b0100,

    /// positive or zero
    PL = 0b0101,

    /// overflow
    VS = 0b0110,

    /// no overflow
    VC = 0b0111,

    //// unsigned higher
    HI = 0b1000,

    /// unsigned lower or same
    LS = 0b1001,

    /// greater or equal
    GE = 0b1010,

    /// less than
    LT = 0b1011,

    /// greater than
    GT = 0b1100,

    /// less than or equal
    LE = 0b1101,

    /// always
    AL = 0b1110,
}

pub trait RegisterIdentifier {
    fn number(&self) -> u8;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rn(pub u8);

impl RegisterIdentifier for Rn {
    fn number(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rd(pub u8);

impl RegisterIdentifier for Rd {
    fn number(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rm(pub u8);

impl RegisterIdentifier for Rm {
    fn number(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rs(pub u8);

impl RegisterIdentifier for Rs {
    fn number(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub enum ShifterOperandCode<Immediate = RotatedImm8, ShiftImm = Imm<5>> {
    Immediate(Immediate),
    ImmediateShift(ShiftedRegister<ShiftImm>),
    RegisterShift(ShiftedRegister<Rs>),
}

#[derive(Debug, PartialEq)]
pub struct ShiftedRegister<Amount, Base = Rm> {
    pub kind: Shift,
    pub amount: Amount,
    pub base: Base,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MultipleAddressingMode {
    DecrementAfter,
    IncrementAfter,
    DecrementBefore,
    IncrementBefore,
}

#[derive(Debug, PartialEq)]
pub struct RegisterList {
    pub registers: [bool; 16],
}

#[derive(Debug, PartialEq)]
pub struct Imm<const N: u32>(u32);

impl<const N: u32> TryFrom<u32> for Imm<N> {
    type Error = UnencodableValueError<u32>;

    fn try_from(value: u32) -> Result<Imm<N>, Self::Error> {
        if value < (2 ^ N) {
            Ok(Imm(value))
        } else {
            Err(UnencodableValueError { value })
        }
    }
}

impl<const N: u32> Imm<N> {
    pub fn new(value: u32) -> Self {
        Self(value)
    }
    pub fn get(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct RotatedImm8 {
    /// 4 bits in instruction
    rotate: u8,
    /// 8 bits in instruction
    value: u8,
}

#[derive(Debug)]
pub struct UnencodableValueError<N> {
    pub value: N,
}

impl TryFrom<u32> for RotatedImm8 {
    type Error = UnencodableValueError<u32>;

    /// tries to create a new RotatedImm8 with the specified value
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        // the quickest way to find an encoding is to find the nearest encodable value <= n, and hope it equals n
        let nearest_below = RotatedImm8::nearest_below(value);

        if nearest_below.get() == value {
            Ok(nearest_below)
        } else {
            Err(UnencodableValueError { value })
        }
    }
}

impl RotatedImm8 {
    pub fn new(value: u8, rotate: u8) -> Self {
        Self { value, rotate }
    }

    /// creates a new RotatedImm8 with the nearest encodable value below or above the one specified, and also returns its offset from the specified value
    pub fn nearest_with_remainder(n: u32) -> (RotatedImm8, i32) {
        let nearest_below = RotatedImm8::nearest_below(n);
        let nearest_above = nearest_below.next();

        let nearest_below_diff = n - nearest_below.get();
        let nearest_above_diff = nearest_above.get() - n;

        if n - nearest_below.get() < nearest_above.get() - n {
            (nearest_below, nearest_below_diff as i32)
        } else {
            (nearest_above, -(nearest_above_diff as i32))
        }
    }

    /// Creates a new RotatedImm8 with the nearest encodable value below the one specified.
    /// If the specified value is encodable, it will encode that value.
    pub fn nearest_below(n: u32) -> RotatedImm8 {
        // we essentially need to choose an 8-bit window to use (window_bottom..window_top)
        // according to the ARM manual, the assembler should choose the smallest rotate possible
        // see page A5-7

        // the number of leading bits
        // capped at 24 because according to the ARM manual, if the number fits in the final 8 bits, we shouldn't rotate
        let leading = u32::min(n.leading_zeros(), 24);

        // the number of trailing zeros
        let trailing = n.trailing_zeros();

        // if there is an 8 bit window that contains all 1s, then we can use that, with the least rotate possible
        // e.g. 0 0|0 0 1 0 0 0 0 1|0 00 0 0 0 0 0 0 1
        //          ^ ^ ^ ^ ^ ^ ^ ^
        // otherwise, we use the least significant window containing the most significant bit
        // e.g. 0 0 0 0|1 0 0 0 0 0 0 0|0 0 0 0 0 0 0 1
        //              ^ ^ ^ ^ ^ ^ ^ ^
        let rotate_amount = u32::min((8 + leading) % 32, 32 - trailing);

        // encode the rotate_amount as a 4 bit value
        // we divide the rotate amount by 2 because ARM only allows rotates by even numbers. this rounds down so it will still capture the most significant 1
        // e.g. 0 0|0 1 0 0 0 0 0 0|0 0 0 0 0 0 0 0 0 0
        //          ^ ^ ^ ^ ^ ^ ^ ^
        let rotate = rotate_amount / 2;

        // get the value by rotating left (working backwards) and taking the least significant byte
        let value = n.rotate_left(rotate * 2) & 0xFF;

        RotatedImm8 {
            // must fit in 8 bits because we mod by 32 and then divide by 2
            rotate: rotate as u8,
            // must fit in 8 bits because we mod by take the least significant byte
            value: value as u8,
        }
    }

    /// calculates the next encodable value
    pub fn next(&self) -> RotatedImm8 {
        let value = self.value.wrapping_add(1);

        // if the value wraps, we need to rotate more to allow a larger value
        if value == 0 {
            let rotate = self.rotate + 1;

            // when rotated by rotate * 2, the 1 will be at the bit above the previous value
            let value = 0b01000000;

            RotatedImm8 { rotate, value }
        } else {
            RotatedImm8 {
                rotate: self.rotate,
                value,
            }
        }
    }

    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn rotate(&self) -> u8 {
        self.rotate
    }

    /// gets the value
    pub fn get(&self) -> u32 {
        (self.value as u32).rotate_right(self.rotate as u32 * 2)
    }
}

#[derive(Debug, PartialEq)]
pub struct SignedImm<const N: u32>(i32);

impl<const N: u32> TryFrom<i32> for SignedImm<N> {
    type Error = UnencodableValueError<i32>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if (-(2 ^ (N as i32 - 1))..(2 ^ (N as i32 - 1))).contains(&value) {
            Ok(SignedImm(value))
        } else {
            Err(UnencodableValueError { value })
        }
    }
}

impl<const N: u32> SignedImm<N> {
    pub fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn get(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub enum BranchKind {
    Branch,
    BranchWithLink,
}

#[derive(Debug, PartialEq)]
pub enum MoveKind {
    Move,
    MoveNot,
}

#[derive(Debug, PartialEq)]
pub enum SetFlags {
    Set,
    DontSet,
}

#[derive(Debug, PartialEq)]
pub enum WriteBack {
    WriteBack,
    NoWriteBack,
}

#[derive(Debug, PartialEq)]
pub enum LoadStoreQuantity {
    Byte,
    Word,
}
