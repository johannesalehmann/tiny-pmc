use crate::UmbError;
use crate::csr::parse_csr;
use indexmap::IndexMap;
use probabilistic_models::Index;
use probabilistic_models::typed_index_collections::Csr;
use std::ops::Range;
use tar::Archive;

/// The files of a decompressed UMB archive. Keeps track of which files have been read, so that
/// unexpected files can be reported once parsing is done.
pub struct UmbArchive<'b> {
    bytes: &'b [u8],
    /// Path -> (location of the file's contents in `bytes`, whether the file has been read)
    files: IndexMap<String, (Range<usize>, bool)>,
}

impl<'b> UmbArchive<'b> {
    /// Creates an archive from the bytes of an (uncompressed) tar file
    pub fn new(bytes: &'b [u8]) -> Result<Self, std::io::Error> {
        let mut files = IndexMap::new();
        let mut archive = Archive::new(bytes);
        for entry in archive.entries()? {
            let entry = entry?;
            // Some archives contain entries for directories
            if entry.header().entry_type().is_dir() {
                continue;
            }
            let start = entry.raw_file_position() as usize;
            let end = start + entry.size() as usize;
            if end > bytes.len() {
                return Err(std::io::ErrorKind::UnexpectedEof.into());
            }
            files.insert(
                entry.path()?.to_string_lossy().into_owned(),
                (start..end, false),
            );
        }
        Ok(Self { bytes, files })
    }

    /// Returns the contents of the file at `path`, if it exists, and marks it as read
    pub fn read_file(&mut self, path: &str) -> Option<&'b [u8]> {
        let (range, read) = self.files.get_mut(path)?;
        *read = true;
        Some(&self.bytes[range.clone()])
    }

    /// Like [`Self::read_file`], but returns an error if the file does not exist
    pub fn read_required_file(&mut self, path: &str) -> Result<&'b [u8], UmbError> {
        self.read_file(path)
            .ok_or_else(|| UmbError::MissingFile(path.into()))
    }

    /// Parses the CSR at `path`, or returns the identity of length `len` if the file does not exist
    pub fn csr_or_identity<From: Index, To: Index>(
        &mut self,
        path: &str,
        len: u64,
    ) -> Result<Csr<From, To>, UmbError> {
        match self.read_file(path) {
            Some(contents) => Ok(parse_csr(contents)?), // TODO: Verify length here?
            None => Ok(Csr::identity(len as usize)),
        }
    }

    /// The paths of all files that have not been read yet, in the order they are stored in
    pub fn unread_files(&self) -> impl Iterator<Item = &str> {
        self.files
            .iter()
            .filter(|(_, (_, read))| !read)
            .map(|(path, _)| path.as_str())
    }
}
