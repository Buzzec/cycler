//! The RwLockCycler uses `parking_lot::RwLock` to keep track of reading and writing to each block without using unsafe code.
//! This is the first implementation but may not be the fastest as locks are required for each switch.
//! Due to the way the data structure is designed there is always an available slot so all lock obtainment are use the try variant.

use std::sync::atomic::AtomicU8;

use parking_lot::RwLock;

pub use builder::{build_single_reader, build_multiple_reader};
pub use reader::RwLockCyclerReader;
pub use writer::RwLockCyclerWriter;

use crate::traits::{EnsureSend, EnsureSync};

mod builder;
mod reader;
mod writer;

#[derive(Debug)]
struct RwLockCycler<T> {
    data_slots: Box<[RwLock<T>]>,
    most_up_to_date: AtomicU8,
}
impl<T> RwLockCycler<T>{
    fn num_readers(&self) -> usize{
        self.data_slots.len() - 2
    }
}
impl<T> EnsureSend for RwLockCycler<T> where T: Send {}
impl<T> EnsureSync for RwLockCycler<T> where T: Send + Sync {}

#[cfg(test)]
mod test {
    use crate::rw_lock_cycler::build_single_reader;
    use crate::test::TestData;
    use std::sync::atomic::Ordering;
    use crate::traits::{WriteAccess, ReadAccess, CyclerWriterDefault, CyclerReader};
    #[test]
    fn default_test() {
        let (mut writer, mut reader) = build_single_reader([TestData::default(), TestData::default(), TestData::default()]);
        assert_eq!(writer.currently_writing, 1);
        assert_eq!(writer.cycler.data_slots.len(), 3);
        assert_eq!(writer.cycler.num_readers(), 1);
        assert_eq!(writer.cycler.most_up_to_date.load(Ordering::SeqCst), 0);
        let new_data = TestData {
            test1: 100,
            test2: "Test2".to_string(),
            test3: Box::new(1002),
        };
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
