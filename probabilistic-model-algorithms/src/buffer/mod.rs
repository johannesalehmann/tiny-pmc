use probabilistic_models::Index;
use probabilistic_models::typed_index_collections::To1;

pub struct Buffer<From: Index, To> {
    values: To1<From, To>,
}

impl<From: Index, To: Default> Buffer<From, To> {
    pub fn from_values(values: To1<From, To>) -> Self {
        Self { values }
    }

    pub fn zero_out(mut self) -> ZeroedBuffer<From, To>
    where
        To: Clone + Default,
    {
        self.values.fill(To::default());
        ZeroedBuffer {
            values: self.values,
        }
    }
}

pub struct ZeroedBuffer<From: Index, To> {
    values: To1<From, To>,
}

impl<From: Index, To> ZeroedBuffer<From, To> {
    pub fn new(len: usize) -> Self
    where
        To: Clone + Default,
    {
        Self {
            values: To1::with_entries(vec![To::default(); len]),
        }
    }

    pub fn from_non_zero_values(values: To1<From, To>) -> Self
    where
        To: Default + Clone,
    {
        let buffer = Buffer::from_values(values);
        buffer.zero_out()
    }

    pub fn into_values(self) -> To1<From, To> {
        self.values
    }
}
