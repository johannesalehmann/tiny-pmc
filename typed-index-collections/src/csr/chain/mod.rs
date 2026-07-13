use crate::{Csr, CsrIterator, Index};

impl<From: Index, Via: Index> Csr<From, Via> {
    pub fn chain<'a, 'b, To: Index>(
        &'a self,
        other: &'b Csr<Via, To>,
    ) -> ChainedCsr<'b, From, Via, To, &'a Csr<From, Via>> {
        ChainedCsr {
            lhs: self,
            rhs: other,
        }
    }
}

pub struct ChainedCsr<'a, From: Copy, Via: Index, To: Index, Lhs: IntoIterator<Item = (From, Via)>>
{
    lhs: Lhs,
    rhs: &'a Csr<Via, To>,
}

impl<'a, From: Copy, Via: Index, To: Index, Lhs: IntoIterator<Item = (From, Via)>>
    ChainedCsr<'a, From, Via, To, Lhs>
{
    pub fn chain<To2: Index>(
        self,
        other: &'a Csr<To, To2>,
    ) -> ChainedCsr<'a, (From, Via), To, To2, ChainedCsr<'a, From, Via, To, Lhs>> {
        ChainedCsr {
            lhs: self,
            rhs: other,
        }
    }
}

impl<'a, From: Copy, Via: Index, To: Index, Lhs: IntoIterator<Item = (From, Via)>> IntoIterator
    for ChainedCsr<'a, From, Via, To, Lhs>
{
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = ChainedCsrIter<From, Via, To, Lhs::IntoIter, CsrIterator<'a, Via, To>>;

    fn into_iter(self) -> Self::IntoIter {
        ChainedCsrIter {
            lhs: self.lhs.into_iter(),
            rhs: self.rhs.into_iter(),
            lhs_element: None,
            rhs_element: None,
        }
    }
}

pub struct ChainedCsrIter<
    From: Copy,
    Via: Index,
    To: Index,
    Lhs: Iterator<Item = (From, Via)>,
    Rhs: Iterator<Item = (Via, To)>,
> {
    lhs: Lhs,
    rhs: Rhs,
    lhs_element: Option<(From, Via)>,
    rhs_element: Option<(Via, To)>,
}

impl<
    From: Copy,
    Via: Index,
    To: Index,
    Lhs: Iterator<Item = (From, Via)>,
    Rhs: Iterator<Item = (Via, To)>,
> Iterator for ChainedCsrIter<From, Via, To, Lhs, Rhs>
{
    type Item = ((From, Via), To);

    fn next(&mut self) -> Option<Self::Item> {
        if self.lhs_element.is_none() {
            self.lhs_element = self.lhs.next();
        }
        if self.rhs_element.is_none() {
            self.rhs_element = self.rhs.next();
        }
        if let Some((via, to)) = self.rhs_element {
            const MESSAGE: &'static str =
                "The left-hand side of the chained Csr ran out of items before the right-hand side";
            let mut lhs = self.lhs_element.as_ref().expect(MESSAGE);
            while lhs.1.raw() < via.raw() {
                self.lhs_element = self.lhs.next();
                lhs = self.lhs_element.as_ref().expect(MESSAGE);
            }
            assert_eq!(lhs.1.raw(), via.raw());
            let res = ((lhs.0, via), to);
            self.rhs_element = self.rhs.next();
            Some(res)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as typed_index_collections;
    use crate::{Csr, Index, index};

    index!(A);
    index!(B);
    index!(C);
    index!(D);

    macro_rules! a {
        ($val: expr) => {
            A::from_raw($val)
        };
    }
    macro_rules! b {
        ($val: expr) => {
            B::from_raw($val)
        };
    }
    macro_rules! c {
        ($val: expr) => {
            C::from_raw($val)
        };
    }
    macro_rules! d {
        ($val: expr) => {
            D::from_raw($val)
        };
    }
    #[test]
    fn simple_test() {
        let csr_1: Csr<A<u64>, B<u64>> = Csr::with_entries(vec![b!(2), b!(3)]);
        let csr_2: Csr<B<u64>, C<u64>> = Csr::with_entries(vec![c!(3), c!(4), c!(7)]);
        let mut iter = csr_1.chain(&csr_2).into_iter();
        assert_eq!(iter.next(), Some(((a!(0), b!(0)), c!(0))));
        assert_eq!(iter.next(), Some(((a!(0), b!(0)), c!(1))));
        assert_eq!(iter.next(), Some(((a!(0), b!(0)), c!(2))));
        assert_eq!(iter.next(), Some(((a!(0), b!(1)), c!(3))));
        assert_eq!(iter.next(), Some(((a!(1), b!(2)), c!(4))));
        assert_eq!(iter.next(), Some(((a!(1), b!(2)), c!(5))));
        assert_eq!(iter.next(), Some(((a!(1), b!(2)), c!(6))));

        assert_eq!(iter.next(), None);
    }
    #[test]
    fn chain_test() {
        let csr_1: Csr<A<u64>, B<u64>> = Csr::with_entries(vec![b!(2), b!(3)]);
        let csr_2: Csr<B<u64>, C<u64>> = Csr::with_entries(vec![c!(3), c!(4), c!(7)]);
        let csr_3: Csr<C<u64>, D<u64>> =
            Csr::with_entries(vec![d![1], d![2], d![3], d![6], d![7], d![8], d![9]]);
        let mut iter = csr_1.chain(&csr_2).chain(&csr_3).into_iter();
        assert_eq!(iter.next(), Some((((a!(0), b!(0)), c!(0)), d!(0))));
        assert_eq!(iter.next(), Some((((a!(0), b!(0)), c!(1)), d!(1))));
        assert_eq!(iter.next(), Some((((a!(0), b!(0)), c!(2)), d!(2))));
        assert_eq!(iter.next(), Some((((a!(0), b!(1)), c!(3)), d!(3))));
        assert_eq!(iter.next(), Some((((a!(0), b!(1)), c!(3)), d!(4))));
        assert_eq!(iter.next(), Some((((a!(0), b!(1)), c!(3)), d!(5))));
        assert_eq!(iter.next(), Some((((a!(1), b!(2)), c!(4)), d!(6))));
        assert_eq!(iter.next(), Some((((a!(1), b!(2)), c!(5)), d!(7))));
        assert_eq!(iter.next(), Some((((a!(1), b!(2)), c!(6)), d!(8))));

        assert_eq!(iter.next(), None);
    }
}
