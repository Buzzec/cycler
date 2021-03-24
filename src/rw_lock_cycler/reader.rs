use std::sync::atomic::Ordering;

use crate::rw_lock_cycler::RwLockCycler;
use parking_lot::{RwLockReadGuard, RwLock};
use crate::traits::{EnsureSend, EnsureSync, ReadAccess, CyclerReader};
use std::mem::swap;

#[cfg(feature = "unsafe_cleanup")]
use crate::static_ref_holder::StaticRefHolder;
#[cfg(feature = "unsafe_cleanup")]
use std::sync::Arc;

/// The reader for a data distributor
#[derive(Debug)]
pub struct RwLockCyclerReader<T> where T: 'static {
    pub(super) cycler: &'static RwLockCycler<T>,
    pub(super) reader: Option<RwLockReadGuard<'static, T>>,
    #[allow(dead_code)]
    #[cfg(feature = "unsafe_cleanup")]
    pub(super) ref_holder: Arc<StaticRefHolder<RwLockCycler<T>>>,
}
impl<T> EnsureSend for RwLockCyclerReader<T> where T: Send + Sync {}
impl<T> EnsureSync for RwLockCyclerReader<T> where T: Send + Sync {}
impl<T> RwLockCyclerReader<T> {
    // /// This changes the reader to read the most up to date version of the data available.
    // /// This may be the same data as the last read or new data.
    // /// This function takes an owned self to ensure nothing is still reading the current block.
    // pub fn read_latest(mut self) -> Self {
    //
    // }
}
impl<T> ReadAccess for RwLockCyclerReader<T> where T: ReadAccess {
    type Read = T::Read;

    #[inline]
    fn read_data(&self) -> &Self::Read {
        // Reader should never be None except inside of read_latest
        self.reader.as_ref().unwrap().read_data()
    }
}
impl<T> CyclerReader<T> for RwLockCyclerReader<T> where T: ReadAccess{
    fn read_latest(&mut self) {
        let mut old_reader = None;
        swap(&mut self.reader, &mut old_reader);
        drop(old_reader);
        let mut most_up_to_date = self.cycler.most_up_to_date.load(Ordering::Relaxed);
        loop {
            if let Some(reader) = RwLock::try_read(&self.cycler.data_slots[most_up_to_date as usize]) {
                self.reader = Some(reader);
                return;
            } else {
                most_up_to_date = self.cycler.most_up_to_date.load(Ordering::Relaxed);
            }
        }
    }
}
