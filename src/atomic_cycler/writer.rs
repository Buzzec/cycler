use std::sync::Arc;
use crate::atomic_cycler::AtomicCycler;
use crate::atomic_rw_lock::AtomicArcWriter;
use std::sync::atomic::AtomicU8;
use crate::{EnsureSend, EnsureSync, ReadAccess, WriteAccess, CyclerWriter, CyclerWriterFn, CyclerWriterMutFn, CyclerWriterDefault};

/// The writer to an `AtomicCyclerWriter`
#[derive(Debug)]
pub struct AtomicCyclerWriter<T> where T: 'static {
    pub(super) cycler: Arc<AtomicCycler<T>>,
    pub(super) writer: AtomicArcWriter<T, AtomicU8>,
    pub(super) currently_writing: u8,
}
impl<T> EnsureSend for AtomicCyclerWriter<T> where T: Send + Sync {}
impl<T> EnsureSync for AtomicCyclerWriter<T> where T: Send + Sync {}
impl<T> ReadAccess for AtomicCyclerWriter<T> where T: ReadAccess {
    type Read = T::Read;

    /// Gets a shared reference to the read data of the current block
    #[inline]
    fn read_data(&self) -> &Self::Read {
        self.writer.read_data()
    }
}
impl<T> WriteAccess for AtomicCyclerWriter<T> where T: WriteAccess {
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
impl<T> CyclerWriter<T> for AtomicCyclerWriter<T> where T: WriteAccess {}
impl<T> CyclerWriterFn<T> for AtomicCyclerWriter<T> where T: WriteAccess {
    fn write_next_fn(&mut self, clone_fn: fn(&mut T, &T)) {
        atomic_cycler_fn!(self, clone_fn);
    }

    fn write_next_fn_impl(&mut self, clone_fn: impl FnOnce(&mut T, &T)) where Self: Sized,
    {
        atomic_cycler_fn!(self, clone_fn);
    }

    fn write_next_fn_dyn(&mut self, clone_fn: &mut dyn FnMut(&mut T, &T)) {
        atomic_cycler_fn!(self, clone_fn);
    }

    fn write_next_fn_dyn_boxed(&mut self, clone_fn: Box<dyn FnOnce(&mut T, &T)>) {
        atomic_cycler_fn!(self, clone_fn);
    }
}
impl<T> CyclerWriterMutFn<T> for AtomicCyclerWriter<T> where T: WriteAccess {
    fn write_next_mut_fn(&mut self, clone_fn: fn(&mut T, &mut T)) {
        atomic_cycler_mut_fn!(self, clone_fn);
    }

    fn write_next_mut_fn_impl(&mut self, clone_fn: impl FnOnce(&mut T, &mut T)) where Self: Sized, {
        atomic_cycler_mut_fn!(self, clone_fn);
    }

    fn write_next_mut_fn_dyn(&mut self, clone_fn: &mut dyn FnMut(&mut T, &mut T)) {
        atomic_cycler_mut_fn!(self, clone_fn);
    }

    fn write_next_mut_fn_dyn_boxed(&mut self, clone_fn: Box<dyn FnOnce(&mut T, &mut T)>) {
        atomic_cycler_mut_fn!(self, clone_fn);
    }
}
impl<T> CyclerWriterDefault<T> for AtomicCyclerWriter<T> where T: Clone + WriteAccess {
    fn write_next(&mut self) {
        self.write_next_fn(T::clone_from)
    }
}
