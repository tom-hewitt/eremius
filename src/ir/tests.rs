use crate::ir::RotatedImm8;

#[test]
fn test_rotated_imm_encoding() {
    let imm = RotatedImm8::try_from(0x0003FC00).unwrap();

    assert_eq!(imm.value(), 0xFF);
    assert_eq!(imm.rotate(), 0xB);
    assert_eq!(imm.get(), 0x0003FC00);
}

#[test]
fn test_rotated_imm_nearest_with_remainder() {
    let (imm, remainder) = RotatedImm8::nearest_with_remainder(0b11000000000000000000000000000011);

    assert_eq!(imm.value(), 0b11000000);
    assert_eq!(imm.rotate(), 4);
    assert_eq!(imm.get(), 0b11000000000000000000000000000000);
    assert_eq!(remainder, 0b11);
}

#[test]
fn test_rotated_imm_nearest_with_negative_remainder() {
    let (imm, remainder) = RotatedImm8::nearest_with_remainder(0b01111111111111111111111111111111);

    assert_eq!(imm.value(), 0b10000000);
    assert_eq!(imm.rotate(), 4);
    assert_eq!(imm.get(), 0b10000000000000000000000000000000);
    assert_eq!(remainder, -1);
}
