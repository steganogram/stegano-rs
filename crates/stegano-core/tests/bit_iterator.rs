use bitstream_io::{BitRead, BitReader, LittleEndian};

use stegano_core::BitIterator;

#[test]
fn should_return_the_8th_bits_of_h_in_little_endian_byte_order() {
    let b = [0b0100_1000, 0b0110_0001, 0b0110_1100];
    let mut it = BitIterator::new(&b[..]);

    assert_eq!(it.next().unwrap(), 0, "1st bit not correct");
    assert_eq!(it.next().unwrap(), 0, "2nd bit not correct");
    assert_eq!(it.next().unwrap(), 0, "3rd bit not correct");
    assert_eq!(it.next().unwrap(), 1, "4th bit not correct");
    assert_eq!(it.next().unwrap(), 0, "5th bit not correct");
    assert_eq!(it.next().unwrap(), 0, "6th bit not correct");
    assert_eq!(it.next().unwrap(), 1, "7th bit not correct");
    assert_eq!(it.next().unwrap(), 0, "8th bit not correct");
}

#[test]
fn should_return_8_bits_of_e_in_little_endian_byte_order_after_skip_8() {
    let b = [0b0100_1000, 0b0110_0001];
    let mut it = BitIterator::new(&b[..]).skip(8);

    assert_eq!(it.next().unwrap(), 1, "1st bit not correct");
    assert_eq!(it.next().unwrap(), 0, "2nd bit not correct");
    assert_eq!(it.next().unwrap(), 0, "3rd bit not correct");
    assert_eq!(it.next().unwrap(), 0, "4th bit not correct");
    assert_eq!(it.next().unwrap(), 0, "5th bit not correct");
    assert_eq!(it.next().unwrap(), 1, "6th bit not correct");
    assert_eq!(it.next().unwrap(), 1, "7th bit not correct");
    assert_eq!(it.next().unwrap(), 0, "8th bit not correct");
    assert_eq!(
        it.next(),
        None,
        "it should end after the last bit on the last byte"
    );
}

#[test]
fn should_behave_as_the_bit_reader() {
    let b = [0b0100_1000, 0b0110_0001];
    let mut it = BitIterator::new(&b[..]);
    let mut reader = BitReader::endian(&b[..], LittleEndian);

    for i in 0..16 {
        assert_eq!(
            it.next().unwrap(),
            if reader.read_bit().unwrap() { 1 } else { 0 },
            "{} bit not correct",
            i
        );
    }
}
