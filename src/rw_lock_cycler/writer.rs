use crate::rw_lock_cycler::RwLockCycler;
use crate::traits::*;
use parking_lot::RwLockWriteGuard;

#[cfg(feature = "unsafe_cleanup")]
use crate::static_ref_holder::StaticRefHolder;
#[cfg(feature = "unsafe_cleanup")]
use std::sync::Arc;

/// The writer to an `RwLockCycler`
#[derive(Debug)]
pub struct RwLockCyclerWriter<T> where T: 'static {
    pub(super) cycler: &'static RwLockCycler<T>,
    pub(super) writer: RwLockWriteGuard<'static, T>,
    pub(super) currently_writing: u8,
    #[allow(dead_code)]
    #[cfg(feature = "unsafe_cleanup")]
    pub(super) ref_holder: Arc<StaticRefHolder<RwLockCycler<T>>>,
}
impl<T> EnsureSend for RwLockCyclerWriter<T> where T: Send + Sync {}
impl<T> EnsureSync for RwLockCyclerWriter<T> where T: Send + Sync {}
impl<T> ReadAccess for RwLockCyclerWriter<T> where T: ReadAccess {
    type Read = T::Read;

    /// Gets a shared reference to the read data of the current block
    #[inline]
    fn read_data(&self) -> &Self::Read {
        self.writer.read_data()
    }
}
impl<T> WriteAccess for RwLockCyclerWriter<T> where T: WriteAccess {
    type Write = T::Write;

    /// Gets a shared reference to the write data of the current block
    #[inline]
    fn write_data(&self) -> &Self::Write {
        self.writer.write_data()
    }

    /// Gets an exclusive reference to the write data of the current block
    #[inline]
    fn write_data_mut(&mut self) -> &mut Self::Write {
        self.writer.write_data_mut()
    }
}
impl<T> CyclerWriter<T> for RwLockCyclerWriter<T> where T: WriteAccess {}
impl<T> CyclerWriterFn<T> for RwLockCyclerWriter<T> where T: WriteAccess {
    fn write_next_fn(&mut self, clone_fn: fn(&mut T, &T)) {
        rw_cycler_fn!(self, clone_fn);
    }

    fn write_next_fn_impl(&mut self, clone_fn: impl FnOnce(&mut T, &T)) where Self: Sized,
    {
        rw_cycler_fn!(self, clone_fn);
    }

    fn write_next_fn_dyn(&mut self, clone_fn: &mut dyn FnMut(&mut T, &T)) {
        rw_cycler_fn!(self, clone_fn);
    }

    fn write_next_fn_dyn_boxed(&mut self, clone_fn: Box<dyn FnOnce(&mut T, &T)>) {
        rw_cycler_fn!(self, clone_fn);
    }
}
impl<T> CyclerWriterMutFn<T> for RwLockCyclerWriter<T> where T: WriteAccess {
    fn write_next_mut_fn(&mut self, clone_fn: fn(&mut T, &mut T)) {
        rw_cycler_mut_fn!(self, clone_fn);
    }

    fn write_next_mut_fn_impl(&mut self, clone_fn: impl FnOnce(&mut T, &mut T)) where Self: Sized,
    {
        rw_cycler_mut_fn!(self, clone_fn);
    }

    fn write_next_mut_fn_dyn(&mut self, clone_fn: &mut dyn FnMut(&mut T, &mut T)) {
        rw_cycler_mut_fn!(self, clone_fn);
    }

    fn write_next_mut_fn_dyn_boxed(&mut self, clone_fn: Box<dyn FnOnce(&mut T, &mut T)>) {
        rw_cycler_mut_fn!(self, clone_fn);
    }
}
impl<T> CyclerWriterDefault<T> for RwLockCyclerWriter<T> where T: Clone + WriteAccess {
    fn write_next(&mut self) {
        self.write_next_fn(T::clone_from)
    }
}
