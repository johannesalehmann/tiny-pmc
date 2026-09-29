use flate2::read::GzDecoder;
use std::io::{Cursor, Read};
use std::path::Path;
use tar::Archive;
use xz2::read::XzDecoder;

#[derive(Clone, Debug, PartialEq)]
enum DecompressionMethod {
    Gzip,
    Xz,
    NotCompressed,
}

#[derive(Debug)]
pub enum InputError {
    IoError(std::io::Error),
    DecompressionError {
        method: DecompressionMethod,
        error: std::io::Error,
    },
    NotATarFile {
        decompression_method: DecompressionMethod,
    },
}

pub fn get_contents<P: AsRef<Path>>(path: P) -> Result<Archive<Cursor<Vec<u8>>>, InputError> {
    let bytes = std::fs::read(path).map_err(InputError::IoError)?;
    let (decompressed_bytes, decompression_method) =
        if bytes.len() >= 3 && &bytes[0..3] == &[0x1F, 0x8B, 0x08] {
            // gzip file
            let mut decoder = GzDecoder::new(&bytes[..]);
            let mut target = Vec::new();
            let _bytes = decoder.read_to_end(&mut target).map_err(|error| {
                InputError::DecompressionError {
                    method: DecompressionMethod::Gzip,
                    error,
                }
            })?;
            (target, DecompressionMethod::Gzip)
        } else if bytes.len() >= 6 && &bytes[0..6] == &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00] {
            // xz compressed
            let mut decoder = XzDecoder::new(&bytes[..]);
            let mut target = Vec::new();
            let _bytes = decoder.read_to_end(&mut target).map_err(|error| {
                InputError::DecompressionError {
                    method: DecompressionMethod::Xz,
                    error,
                }
            })?;
            (target, DecompressionMethod::Xz)
        } else {
            (bytes, DecompressionMethod::NotCompressed)
        };
    if decompressed_bytes.len() >= 262
        && &decompressed_bytes[257..262] == &[0x75, 0x73, 0x74, 0x61, 0x72]
    {
        let archive = Archive::new(Cursor::new(decompressed_bytes));
        Ok(archive)
    } else {
        Err(InputError::NotATarFile {
            decompression_method,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::file_input::get_contents;

    fn test_model(path: &str) {
        let mut archive = get_contents(path).unwrap();
        let entries = archive
            .entries()
            .unwrap()
            .map(|e| e.unwrap().path().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            entries,
            vec![
                "index.json",
                "state-to-choices.bin",
                "choice-to-branches.bin",
                "branch-to-probability.bin",
                "branch-to-target.bin",
                "state-is-initial.bin",
                "actions/choices/string-mapping.bin",
                "actions/choices/strings.bin",
                "actions/choices/values.bin",
                "annotations/aps/g/states/values.bin",
                "annotations/rewards/r/choices/values.bin",
                "valuations/states/valuations.bin",
            ]
        )
    }

    #[test]
    fn uncompressed() {
        test_model("examples/from_umb_spec/mdp-uncompressed.umb")
    }
    #[test]
    fn gzip() {
        test_model("examples/from_umb_spec/mdp-gzip.umb")
    }
    #[test]
    fn xz() {
        test_model("examples/from_umb_spec/mdp-xz.umb")
    }
}
