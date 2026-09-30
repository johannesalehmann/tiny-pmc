use num_traits::ToPrimitive;
use num_traits::bounds::Bounded;
use probabilistic_models::typed_index_collections::Csr;
use probabilistic_models::{Index, RawIndex};
use std::io::Read;

fn read_u64<R: Read>(mut reader: R, buffer: &mut [u8; 8]) -> Result<Option<u64>, std::io::Error> {
    let first_byte = reader.read_exact(&mut buffer[0..1]);
    match first_byte {
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
        Err(err) => Err(err),
        Ok(()) => {
            reader.read_exact(&mut buffer[1..8])?;
            Ok(Some(u64::from_le_bytes(*buffer)))
        }
    }
}

#[derive(Debug)]
pub enum CsrError {
    IoError(std::io::Error),
    NoFirstEntry,
    FirstEntryNonZero {
        value: u64,
    },
    IndexTypeTooNarrow {
        value: u64,
        max_allowed: u64,
    },
    NotMonotonic {
        index: usize,
        previous: u64,
        value: u64,
    },
}

impl From<std::io::Error> for CsrError {
    fn from(value: std::io::Error) -> Self {
        CsrError::IoError(value)
    }
}

pub fn parse_csr<From: Index, To: Index, R: Read>(
    mut reader: R,
) -> Result<Csr<From, To>, CsrError> {
    let mut u8_buffer = [0u8; 8];
    let mut values = Vec::new();
    let mut previous = 0;
    let first_value = read_u64(&mut reader, &mut u8_buffer)?;
    match first_value {
        None => Err(CsrError::NoFirstEntry),
        Some(value) if value > 0 => Err(CsrError::FirstEntryNonZero { value }),
        _ => Ok(()),
    }?;
    while let Some(value) = read_u64(&mut reader, &mut u8_buffer)? {
        if value < previous {
            return Err(CsrError::NotMonotonic {
                index: values.len() + 1,
                previous,
                value,
            });
        }
        previous = value;
        match To::RawType::try_from_usize(value as usize) {
            Some(index) => values.push(To::from_raw(index)),
            None => {
                return Err(CsrError::IndexTypeTooNarrow {
                    value,
                    max_allowed: To::RawType::max_value().to_u64().unwrap(),
                });
            }
        }
    }

    Ok(Csr::with_entries(values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use probabilistic_models::{ChoiceIndex, StateIndex};

    fn bytes(values: &[u64]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    fn parse<To: Index>(data: Vec<u8>) -> Result<Csr<StateIndex<u32>, To>, CsrError> {
        parse_csr(data.as_slice())
    }

    #[test]
    fn parses_entries_without_leading_zero() {
        let csr = parse::<ChoiceIndex<u32>>(bytes(&[0, 2, 2, 5])).unwrap();
        let expected: Vec<_> = [2, 2, 5].map(ChoiceIndex::from_raw).into();
        assert_eq!(csr, Csr::with_entries(expected));
    }

    #[test]
    fn empty_csr() {
        assert!(parse::<ChoiceIndex<u32>>(bytes(&[0])).unwrap().is_empty());
    }

    #[test]
    fn no_first_entry() {
        assert!(matches!(
            parse::<ChoiceIndex<u32>>(vec![]),
            Err(CsrError::NoFirstEntry)
        ));
    }

    #[test]
    fn nonzero_first_entry() {
        assert!(matches!(
            parse::<ChoiceIndex<u32>>(bytes(&[1, 2])),
            Err(CsrError::FirstEntryNonZero { value: 1 })
        ));
    }

    #[test]
    fn value_too_large_for_index() {
        assert!(matches!(
            parse::<ChoiceIndex<u8>>(bytes(&[0, 255, 256])),
            Err(CsrError::IndexTypeTooNarrow {
                value: 256,
                max_allowed: 255
            })
        ));
    }

    #[test]
    fn nonmonotonic_indices() {
        assert!(matches!(
            parse::<ChoiceIndex<u32>>(bytes(&[0, 3, 2])),
            Err(CsrError::NotMonotonic {
                index: 2,
                previous: 3,
                value: 2
            })
        ));
    }

    #[test]
    fn max_value_of_index_type_is_accepted() {
        assert!(parse::<ChoiceIndex<u8>>(bytes(&[0, 255])).is_ok());
    }

    #[test]
    fn truncated_entry_is_io_error() {
        let mut data = bytes(&[0, 1]);
        data.truncate(12);
        assert!(matches!(
            parse::<ChoiceIndex<u32>>(data),
            Err(CsrError::IoError(_))
        ));
    }

    #[test]
    fn truncated_first_entry_is_io_error() {
        assert!(matches!(
            parse::<ChoiceIndex<u32>>(vec![0; 4]),
            Err(CsrError::IoError(_))
        ));
    }
}
