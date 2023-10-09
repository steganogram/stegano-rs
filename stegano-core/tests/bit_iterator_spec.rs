use bitstream_io::{BitRead, BitReader, LittleEndian};
use speculate::speculate;

use stegano_core::BitIterator;

speculate! {
    describe "BitIterator::next()" {
        // String: H           e           l
        // Hex   : 0x48        0x61        0x6C
        // Binary: 0b01001000  0b01100001  0b01101100
        it "should return the 8 bits of 'H' in LittleEndian byte order" {
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

        // String: H           e           l
        // Hex   : 0x48        0x61        0x6C
        // Binary: 0b01001000  0b01100001  0b01101100
        it "should return 8 bits of 'e' in LittleEndian byte order after skip(8)" {
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
            assert_eq!(it.next(), None, "it should end after the last bit on the last byte");
        }

        it "should behave as the BitReader" {
            let b = [0b0100_1000, 0b0110_0001];
            let mut it = BitIterator::new(&b[..]);
            let mut reader = BitReader::endian(
                &b[..],
                LittleEndian
            );

            for i in 0..16 {
                assert_eq!(
                    it.next().unwrap(),
                    if reader.read_bit().unwrap() { 1 } else { 0 },
                    "{} bit not correct", i
                );
            }
        }
    }
}
