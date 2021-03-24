use crate::rw_lock_cycler::RwLockCycler;
use parking_lot::RwLockWriteGuard;
use crate::traits::*;

#[cfg(feature = "unsafe_cleanup")]
use crate::static_ref_holder::StaticRefHolder;
#[cfg(feature = "unsafe_cleanup")]
use std::sync::Arc;

/// The writer to a data distributor
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
impl<T> RwLockCyclerWriter<T> {
    // clone_fn!{
    //     "Changes the current writing block to another that is available copying from the last block using `clone_fn`.
    //      `clone_fn` is the same signature as `Clone::clone_from`, the first argument is what is being cloned to and the second being what is cloned from.
    //      This version takes an `impl FnOnce` so has no runtime cost for virtual lookup.",
    //     write_next_clone_fn,
    //     impl FnOnce(&mut T, &T)
    // }
    //
    // clone_fn!{
    //     "Changes the current writing block to another that is available copying from the last block using `clone_fn`.
    //      `clone_fn` is the same signature as `Clone::clone_from`, the first argument is what is being cloned to and the second being what is cloned from.
    //      This version takes a `&dyn Fn` allowing for dynamic functions",
    //     write_next_dyn_clone_fn,
    //     &dyn Fn(&mut T, &T)
    // }
    //
    // clone_fn!{
    //     "Changes the current writing block to another that is available copying from the last block using `clone_fn`.
    //      `clone_fn` is the same signature as `Clone::clone_from`, the first argument is what is being cloned to and the second being what is cloned from.
    //      This version takes a `&mut dyn FnMut` allowing for dynamic functions",
    //     write_next_dyn_mut_clone_fn,
    //     &mut dyn FnMut(&mut T, &T)
    // }
    //
    // clone_fn!{
    //     "Changes the current writing block to another that is available copying from the last block using `clone_fn`.
    //      `clone_fn` is the same signature as `Clone::clone_from`, the first argument is what is being cloned to and the second being what is cloned from.
    //      This version takes a `Box<dyn FnOnce>` allowing for dynamic functions",
    //     write_next_boxed_clone_fn,
    //     Box<dyn FnOnce(&mut T, &T)>
    // }

}
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
impl<T> CyclerWriter<T> for RwLockCyclerWriter<T> where T: WriteAccess{}
impl<T> CyclerWriterFn<T> for RwLockCyclerWriter<T> where T: WriteAccess{
    fn write_next_fn(&mut self, clone_fn: fn(&mut T, &T)) {
        rw_cycler_fn!(self, clone_fn);
    }

    fn write_next_fn_impl(&mut self, clone_fn: impl FnOnce(&mut T, &T)) where Self: Sized {
        rw_cycler_fn!(self, clone_fn);
    }

    fn write_next_fn_dyn(&mut self, clone_fn: &mut dyn FnMut(&mut T, &T)) {
        rw_cycler_fn!(self, clone_fn);
    }

    fn write_next_fn_dyn_boxed(&mut self, clone_fn: Box<dyn FnOnce(&mut T, &T)>) {
        rw_cycler_fn!(self, clone_fn);
    }
}
impl<T> CyclerWriterMutFn<T> for RwLockCyclerWriter<T> where T: WriteAccess{
    fn write_next_mut_fn(&mut self, clone_fn: fn(&mut T, &mut T)) {
        rw_cycler_mut_fn!(self, clone_fn);
    }

    fn write_next_mut_fn_impl(&mut self, clone_fn: impl FnOnce(&mut T, &mut T)) where Self: Sized {
        rw_cycler_mut_fn!(self, clone_fn);
    }

    fn write_next_mut_fn_dyn(&mut self, clone_fn: &mut dyn FnMut(&mut T, &mut T)) {
        rw_cycler_mut_fn!(self, clone_fn);
    }

    fn write_next_mut_fn_dyn_boxed(&mut self, clone_fn: Box<dyn FnOnce(&mut T, &mut T)>) {
        rw_cycler_mut_fn!(self, clone_fn);
    }
}
impl<T> CyclerWriterDefault<T> for RwLockCyclerWriter<T> where T: Clone + WriteAccess{
    fn write_next(&mut self) {
        self.write_next_fn(T::clone_from)
    }
}
