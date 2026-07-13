mod csr;
pub use csr::chain::{ChainedCsr, ChainedCsrIter};
pub use csr::{
    Csr, CsrIterator, CsrRangeIterator, CsrRanges, CsrRangesIterator, EnumeratingCsrRangesIterator,
};

mod named_collection;
pub use named_collection::NamedTo1;

mod to1;
pub use to1::*;

mod index;
pub use index::{Index, IndexRange, IndexRangeIterator, RawIndex, SemiboundedIndexRange};

// TODO: Extract the key behaviours or Csr and To1 into traits. Then new versions can be constructed
//  by chaining Csrs, using To1 to map a Csr to a new target, zipping two To1s. These can then be
//  used in place of standard Csrs and To1.
//  This also enables Identity maps, which behave like Csrs, but map everything to a one-sized
//  range.
