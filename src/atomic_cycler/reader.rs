use std::sync::Arc;
use crate::atomic_cycler::AtomicCycler;
use crate::{EnsureSend, EnsureSync, ReadAccess, CyclerReader};
use crate::atomic_rw_lock::AtomicArcReader;
use std::sync::atomic::{AtomicU8, Ordering};

/// The reader for an `RwLockCycler`
#[derive(Debug)]
pub struct AtomicCyclerReader<T>{
    pub(super) cycler: Arc<AtomicCycler<T>>,
    pub(super) reader: Option<AtomicArcReader<T, AtomicU8>>,
}
impl<T> EnsureSend for AtomicCyclerReader<T> where T: Send + Sync{}
impl<T> EnsureSync for AtomicCyclerReader<T> where T: Send + Sync{}
impl<T> ReadAccess for AtomicCyclerReader<T> where T: ReadAccess{
    type Read = T::Read;

    fn read_data(&self) -> &Self::Read {
        self.reader.as_ref().unwrap().read_data()
    }
}
impl<T> CyclerReader<T> for AtomicCyclerReader<T> where T: ReadAccess{
    fn read_latest(&mut self) {
        drop(self.reader.take());
        let mut most_up_to_date = self.cycler.most_up_to_date.load(Ordering::Relaxed);
        loop{
            if let Some(reader) = self.cycler.data_slots[most_up_to_date as usize].try_read_static(){
                self.reader = Some(reader);
                return;
            }
            else{
                most_up_to_date = self.cycler.most_up_to_date.load(Ordering::Relaxed);
            }
        }
    }
}
