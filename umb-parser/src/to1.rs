use probabilistic_models::Index;
use probabilistic_models::typed_index_collections::To1;
use std::convert::Infallible;
use std::io::Read;

pub trait FromLeBytes: Sized {
    /// Number of bytes of one encoded element.
    const SIZE: usize;
    type Error;

    /// Decodes one element. `bytes` always has length `Self::SIZE`.
    fn from_le_bytes(bytes: &[u8]) -> Result<Self, Self::Error>;
}

macro_rules! impl_from_le_bytes {
    ($($t:ty),*) => {$(
        impl FromLeBytes for $t {
            const SIZE: usize = size_of::<$t>();
            type Error = Infallible;

            fn from_le_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
                Ok(<$t>::from_le_bytes(bytes.try_into().unwrap()))
            }
        }
    )*};
}
impl_from_le_bytes!(u8, u16, u32, u64, i8, i16, i32, i64, f64);

#[derive(Debug)]
pub enum To1Error<E> {
    IoError(std::io::Error),
    InvalidElement(E),
}

impl<E> From<std::io::Error> for To1Error<E> {
    fn from(value: std::io::Error) -> Self {
        To1Error::IoError(value)
    }
}

pub fn parse_to1<From: Index, T: FromLeBytes, R: Read>(
    mut reader: R,
) -> Result<To1<From, T>, To1Error<T::Error>> {
    let mut buffer = vec![0u8; T::SIZE];
    let mut values = Vec::new();
    loop {
        match reader.read_exact(&mut buffer[..1]) {
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(err.into()),
            Ok(()) => {}
        }
        reader.read_exact(&mut buffer[1..])?;
        values.push(T::from_le_bytes(&buffer).map_err(To1Error::InvalidElement)?);
    }
    Ok(To1::with_entries(values))
}

#[derive(Debug)]
pub enum BoolTo1Error {
    IoError(std::io::Error),
    SizeMismatch { expected: usize, actual: usize },
    NonZeroPadding,
}

impl From<std::io::Error> for BoolTo1Error {
    fn from(value: std::io::Error) -> Self {
        BoolTo1Error::IoError(value)
    }
}

pub fn parse_to1_bool<From: Index, R: Read>(
    mut reader: R,
    len: usize,
) -> Result<To1<From, bool>, BoolTo1Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    let expected = len.div_ceil(64) * 8;
    if bytes.len() != expected {
        return Err(BoolTo1Error::SizeMismatch {
            expected,
            actual: bytes.len(),
        });
    }
    let bit = |i: usize| bytes[i / 8] >> (i % 8) & 1 == 1;
    if (len..expected * 8).any(bit) {
        return Err(BoolTo1Error::NonZeroPadding);
    }
    Ok(To1::with_entries((0..len).map(bit).collect::<Vec<_>>()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use probabilistic_models::StateIndex;

    type S = StateIndex<u32>;

    fn bytes(values: &[u64]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    #[test]
    fn parses_elements() {
        let to1: To1<S, u64> = parse_to1(bytes(&[3, 0, 7]).as_slice()).unwrap();
        assert_eq!(to1.entries(), [3, 0, 7]);
    }

    #[test]
    fn parses_little_endian_u16() {
        let to1: To1<S, u16> = parse_to1([1, 0, 0, 1].as_slice()).unwrap();
        assert_eq!(to1.entries(), [1, 256]);
    }

    #[test]
    fn empty_input_gives_empty_to1() {
        let to1: To1<S, u32> = parse_to1([].as_slice()).unwrap();
        assert!(to1.is_empty());
    }

    #[test]
    fn truncated_element_is_io_error() {
        let result = parse_to1::<S, u32, _>([0u8; 6].as_slice());
        assert!(matches!(result, Err(To1Error::IoError(_))));
    }

    #[test]
    fn parses_bitset_across_words() {
        let to1: To1<S, bool> = parse_to1_bool(bytes(&[0b101, 0b10]).as_slice(), 66).unwrap();
        let true_indices: Vec<_> = (0..66).filter(|&i| to1.entries()[i]).collect();
        assert_eq!(true_indices, [0, 2, 65]);
    }

    #[test]
    fn empty_bitset_has_no_bytes() {
        let to1: To1<S, bool> = parse_to1_bool([].as_slice(), 0).unwrap();
        assert!(to1.is_empty());
    }

    #[test]
    fn bitset_size_must_match_length() {
        assert!(matches!(
            parse_to1_bool::<S, _>(bytes(&[0]).as_slice(), 65),
            Err(BoolTo1Error::SizeMismatch {
                expected: 16,
                actual: 8
            })
        ));
    }

    #[test]
    fn set_padding_bit_is_rejected() {
        assert!(matches!(
            parse_to1_bool::<S, _>(bytes(&[0b100]).as_slice(), 2),
            Err(BoolTo1Error::NonZeroPadding)
        ));
    }
}
