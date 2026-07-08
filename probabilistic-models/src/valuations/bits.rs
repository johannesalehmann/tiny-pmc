use std::ops::{IndexMut, Range};

pub trait GetBits {
    fn bits(&self, range: Range<usize>) -> u64;
    fn bit(&self, offset: usize) -> bool;
}

pub trait SetBits {
    fn set_bits(&mut self, range: Range<usize>, bits: u64);
    fn set_bit(&mut self, range: usize, value: bool);
}

trait PrimitiveBitSource: std::ops::Not<Output = Self> + num_traits::PrimInt + Into<u64> {
    fn from_u64(val: u64) -> Self;
}

impl PrimitiveBitSource for u8 {
    fn from_u64(val: u64) -> Self {
        val as u8
    }
}
impl PrimitiveBitSource for u16 {
    fn from_u64(val: u64) -> Self {
        val as u16
    }
}
impl PrimitiveBitSource for u32 {
    fn from_u64(val: u64) -> Self {
        val as u32
    }
}
impl PrimitiveBitSource for u64 {
    fn from_u64(val: u64) -> Self {
        val
    }
}

impl<I: PrimitiveBitSource> GetBits for I {
    fn bits(&self, range: Range<usize>) -> u64 {
        let mask = get_mask(range.clone());
        let value = *self & mask;
        (value >> range.start).into()
    }

    fn bit(&self, offset: usize) -> bool {
        (*self & (Self::one() << offset)) != Self::zero()
    }
}
impl<I: PrimitiveBitSource> SetBits for I {
    fn set_bits(&mut self, range: Range<usize>, bits: u64) {
        let mask: I = get_mask(range.clone());
        *self = (*self & !mask) | ((Self::from_u64(bits) << range.start) & mask)
    }

    fn set_bit(&mut self, offset: usize, value: bool) {
        let bit = I::one() << offset;
        if value {
            *self = *self | bit
        } else {
            *self = (*self & !bit)
        }
    }
}

impl GetBits for &[u64] {
    fn bits(&self, range: Range<usize>) -> u64 {
        let start_field = range.start >> 6;
        let end_field = range.end >> 6;
        let start_index = range.start & 0b111111;
        let end_index = range.end & 0b111111;

        if start_field == end_field {
            let mask: u64 = get_mask(start_index..end_index);
            (self[start_field] & mask) >> start_index
        } else if start_field + 1 == end_field && end_index == 0 {
            let mask: u64 = get_mask(start_index..64);
            (self[start_field] & mask) >> start_index
        } else if start_field + 1 == end_field {
            let start_mask: u64 = get_mask(start_index..64);
            let end_mask: u64 = get_mask(0..end_index);
            let start_component = (self[start_field] & start_mask) >> start_index;
            let end_component = self[end_field] & end_mask;

            start_component | (end_component << (64 - start_index))
        } else {
            panic!(
                "Cannot access variable values if start and end offset differ by more than 64 bits."
            )
        }
    }

    fn bit(&self, offset: usize) -> bool {
        let field = offset >> 6;
        let index = offset & 0b111111;
        (self[field] & (1 << index)) != 0
    }
}
impl SetBits for &mut [u64] {
    fn set_bits(&mut self, range: Range<usize>, bits: u64) {
        let start_field = range.start >> 6;
        let end_field = range.end >> 6;
        let start_index = range.start & 0b111111;
        let end_index = range.end & 0b111111;
        if start_field == end_field {
            let mask: u64 = get_mask(start_index..end_index);
            self[start_field] = (self[start_field] & !mask) | bits << start_index;
        } else if start_field + 1 == end_field && end_index == 0 {
            let mask: u64 = get_mask(start_index..64);
            self[start_field] = (self[start_field] & !mask) | bits << start_index;
        } else if start_field + 1 == end_field {
            let start_mask: u64 = get_mask(start_index..64);
            let end_mask: u64 = get_mask(0..end_index);
            let start_component = bits & get_mask::<u64>(0..(64 - start_index));
            let end_component =
                (bits & get_mask::<u64>((64 - start_index)..range.len())) >> (64 - start_index);

            self[start_field] =
                (self[start_field] & !start_mask) | (start_component << start_index);
            self[end_field] = (self[end_field] & !end_mask) | (end_component);
        } else {
            panic!(
                "Cannot modify variable values if start and end offset differ by more than 64 bits."
            )
        }
    }

    fn set_bit(&mut self, offset: usize, value: bool) {
        let field = offset >> 6;
        let index = offset & 0b111111;
        let bit = 1 << index;
        if value {
            self[field] = self[field] | bit
        } else {
            self[field] = (self[field] & !bit)
        }
    }
}

fn get_mask<I: std::ops::Not<Output = I> + num_traits::PrimInt>(range: Range<usize>) -> I {
    let length = size_of::<I>() * 8;
    let start_mask = if range.start < length {
        (I::one() << range.start) - I::one()
    } else {
        !I::zero()
    };
    let end_mask = if range.end < length {
        (I::one() << range.end) - I::one()
    } else {
        !I::zero()
    };
    end_mask & !start_mask
}

#[cfg(test)]
mod tests {
    use super::{GetBits, SetBits};
    macro_rules! test_value {
        ($name: ident, $int_name: ident, $length: expr) => {
            test_value_with_filler!(
                $name _all_zeroes,
                $int_name,
                $length,
                0
            );
            test_value_with_filler!(
                $name _all_ones,
                $int_name,
                $length,
                !0
            );
            test_value_with_filler!(
                $name _mixed_zeroes_ones,
                $int_name,
                $length,
                (!0) / 3
            );
        };
    }

    macro_rules! test_value_with_filler {
        ($name: ident $suffix: ident, $int_name: ident, $length: expr, $filler: expr) => {
            paste::paste!{
                 #[test]
                fn [<bit_retrieval_ $name $suffix>]() {
                    let type_length = $length;
                    for (value, length) in [
                        (0, 0),
                        (0, 1),
                        (1, 1),
                        (0b1, 1),
                        (0b101, 3),
                        (0b1101, 4),
                        (0b0010, 4),
                        (0b101110, 6),
                    ] {
                        for offset in 0..(type_length - length) {
                            let mut base = 0;
                            for i in 0..type_length {
                                if i < offset || i >= offset + length {
                                    base = base | 1<< i;
                                }
                            }
                            base = base & $filler;
                            let number: $int_name = (value << offset) | base;
                            // Check whether set bits works correctly
                            {
                                let mut set_bits_test = base.clone();
                                set_bits_test.set_bits(offset..offset + length, value as u64);
                                assert_eq!(
                                    number,
                                    set_bits_test,
                                    "set_bits returned incorrect value (got {:b}, expected {:b}) (at offset {})",
                                    number,
                                    set_bits_test,
                                    offset
                                );
                            }
                            // Check whether get_bits works correctly
                            {
                                let bits = number.bits(offset..offset + length);
                                assert_eq!(
                                    bits, value as u64,
                                    "Value {:b} was not correctly retrieved from bit vector {:b} (at offset {})",
                                    value, number, offset
                                );
                            }

                            if length == 1 {
                                let mut set_bit_test = base.clone();
                                set_bit_test.set_bit(offset, value == 1);
                                assert_eq!(number, set_bit_test, "set_bit failed");

                                assert_eq!(number.bit(offset), value == 1, "getting bit failed")
                            }
                        }
                    }
                }
            }
        };
    }

    test_value!(u8, u8, 8);
    test_value!(u16, u16, 16);
    test_value!(u32, u32, 32);
    test_value!(u64, u64, 64);

    #[test]
    fn test_slice() {
        let base_slice: [u64; 4] = [
            0xABCDEF12EF36DB03,
            0xFEDE6418,
            0xFFFFFFFFFFFFFFFF,
            0x12345678,
        ];
        for (value, length) in [
            (0, 0),
            (0, 1),
            (1, 1),
            (0b1, 1),
            (0b101, 3),
            (0b1101, 4),
            (0b0010, 4),
            (0b101110, 6),
        ] {
            for offset in 0..4 * 64 - length {
                let mut values = base_slice.clone();
                for i in 0..length {
                    let location = offset + i;
                    let slice = location >> 6;
                    let in_slice = location & 0b111111;
                    let value_bit = (value & (1 << i)) != 0;
                    let value_bit = if value_bit { 1 } else { 0 };
                    values[slice] = (values[slice] & !(1 << in_slice)) | (value_bit << in_slice);
                }

                // Check whether set_bits works
                {
                    let mut set_bits_test = base_slice.clone();
                    let mut reference = &mut set_bits_test[..];
                    reference.set_bits(offset..offset + length, value);
                    assert_eq!(
                        set_bits_test,
                        values,
                        "set_bits returned incorrect value when setting {:b} at offset {offset}:\n{:#066b} (expected[0])\n{:#066b} (found[0])\n{:#066b} (expected[1])\n{:#066b} (found[1])\n{:#066b} (expected[2])\n{:#066b} (found[2])\n{:#066b} (expected[3])\n{:#066b} (found[3])",
                        value,
                        values[0],
                        set_bits_test[0],
                        values[1],
                        set_bits_test[1],
                        values[2],
                        set_bits_test[2],
                        values[3],
                        set_bits_test[3]
                    );
                }
                // Check whether get_bits works
                {
                    let bits = (&values[..]).bits(offset..offset + length);
                    assert_eq!(bits, value);
                }

                if length == 1 {
                    let mut set_bit_test = base_slice.clone();
                    let mut reference = &mut set_bit_test[..];
                    reference.set_bit(offset, value == 1);
                    assert_eq!(set_bit_test, values);

                    assert_eq!((&values[..]).bit(offset), value == 1)
                }
            }
        }
    }
}
