//! Contains tests for the cycler systems

use crate::traits::{ReadAccess, WriteAccess};

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct TestData {
    pub test1: usize,
    pub test2: String,
    pub test3: Box<usize>,
}
impl Default for TestData {
    fn default() -> Self {
        Self {
            test1: 0,
            test2: "Start".to_string(),
            test3: Box::new(100),
        }
    }
}
impl ReadAccess for TestData {
    type Read = Self;

    fn read_data(&self) -> &Self::Read {
        self
    }
}
impl WriteAccess for TestData {
    type Write = Self;

    fn write_data(&self) -> &Self::Write {
        self
    }

    fn write_data_mut(&mut self) -> &mut Self::Write {
        self
    }
}


