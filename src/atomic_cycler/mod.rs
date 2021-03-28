//! The `AtomicCycler` uses a custom atomic read/write lockless lock to keep track of reading and writing to each block without using unsafe code.
//! This implementation is faster than `RwLockCycler` but it relies on a custom lock that is filled with unsafe code.

mod builder;
mod reader;
mod writer;

pub use builder::{build_multiple_reader, build_single_reader};
pub use reader::AtomicCyclerReader;
pub use writer::AtomicCyclerWriter;

use std::sync::atomic::AtomicU8;
use crate::{EnsureSend, EnsureSync};
use crate::atomic_rw_lock::AtomicRwLock;
use std::sync::Arc;

#[derive(Debug)]
struct AtomicCycler<T> {
    data_slots: Box<[Arc<AtomicRwLock<T, AtomicU8>>]>,
    most_up_to_date: AtomicU8,
}
impl<T> AtomicCycler<T> {
    const fn num_readers(&self) -> usize {
        self.data_slots.len() - 2
    }
}
impl<T> EnsureSend for AtomicCycler<T> where T: Send {}
impl<T> EnsureSync for AtomicCycler<T> where T: Send + Sync {}

#[cfg(test)]
mod test {
    use crate::atomic_cycler::build_single_reader;
    use crate::test::TestData;
    use std::sync::atomic::Ordering;
    use crate::{WriteAccess, ReadAccess, CyclerWriterDefault, CyclerReader};

    #[test]
    fn default_test() {
        let (mut writer, mut reader) =
            build_single_reader([TestData::default(), TestData::default(), TestData::default()]);
        assert_eq!(writer.currently_writing, 1);
        assert_eq!(writer.cycler.data_slots.len(), 3);
        assert_eq!(writer.cycler.num_readers(), 1);
        assert_eq!(writer.cycler.most_up_to_date.load(Ordering::SeqCst), 0);
        let new_data = TestData { test1: 100, test2: "Test2".to_string(), test3: Box::new(1002) };
        writer.write_data_mut().clone_from(&new_data);
        assert_eq!(*reader.read_data(), TestData::default());
        writer.write_next();
        assert_eq!(writer.cycler.most_up_to_date.load(Ordering::SeqCst), 1);
        assert_eq!(writer.currently_writing, 2);
        assert_eq!(*writer.write_data(), new_data);
        assert_eq!(*reader.read_data(), TestData::default());
        reader.read_latest();
        assert_eq!(*reader.read_data(), new_data);
        reader.read_latest();
        assert_eq!(*reader.read_data(), new_data);
    }
}
