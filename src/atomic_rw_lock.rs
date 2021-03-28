#![allow(dead_code)]
//! Allowing dead code here because this may be broken out into its own crate later.
//!
use std::cell::UnsafeCell;
use std::ops::{Add, Deref, DerefMut};
use std::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use crate::{EnsureSend, EnsureSync};
use std::fmt::Debug;

pub type AtomicArcReader<T, A> = AtomicRwLockReader<T, A, Arc<AtomicRwLock<T, A>>>;
pub type AtomicRefReader<'a, T, A> = AtomicRwLockReader<T, A, &'a AtomicRwLock<T, A>>;
pub type AtomicArcWriter<T, A> = AtomicRwLockWriter<T, A, Arc<AtomicRwLock<T, A>>>;
pub type AtomicRefWriter<'a, T, A> = AtomicRwLockWriter<T, A, &'a AtomicRwLock<T, A>>;

#[derive(Debug)]
pub struct AtomicRwLock<T, A> {
    /// Atomic that stores 1 + num readers, or 0 if writing
    count: A,
    data: UnsafeCell<T>,
}
impl<T, A> EnsureSend for AtomicRwLock<T, A> where T: Send, A: Send, {}
unsafe impl<T, A> Sync for AtomicRwLock<T, A> where T: Send, A: Sync, {}
impl<T> AtomicRwLock<T, AtomicU8>{
    pub fn new_u8(value: T) -> Self{
        Self::new(value)
    }
}
impl<T> AtomicRwLock<T, AtomicU16>{
    pub fn new_u16(value: T) -> Self{
        Self::new(value)
    }
}
impl<T> AtomicRwLock<T, AtomicU32>{
    pub fn new_u32(value: T) -> Self{
        Self::new(value)
    }
}
impl<T> AtomicRwLock<T, AtomicU64>{
    pub fn new_u64(value: T) -> Self{
        Self::new(value)
    }
}
impl<T> AtomicRwLock<T, AtomicUsize>{
    pub fn new_usize(value: T) -> Self{
        Self::new(value)
    }
}
impl<T, A> AtomicRwLock<T, A> where A: AtomicValue, {
    pub fn new(value: T) -> Self {
        Self { count: A::new(A::ONE), data: UnsafeCell::new(value) }
    }

    pub fn into_inner(self) -> T{
        self.data.into_inner()
    }
    pub fn get_mut(&mut self) -> &mut T{
        self.data.get_mut()
    }

    fn lock_read(&self) -> bool{
        let mut prev_readers = self.count.load(Ordering::Relaxed);
        loop {
            if prev_readers != A::ZERO {
                match self.count.compare_exchange_weak(prev_readers, prev_readers + A::ONE, Ordering::SeqCst, Ordering::SeqCst) {
                    Ok(_) => return true,
                    Err(new_val) => prev_readers = new_val,
                }
            } else {
                return false;
            }
        }
    }

    fn lock_write(&self) -> bool{
        let mut prev_readers = self.count.load(Ordering::Relaxed);
        loop{
            if prev_readers == A::ONE{
                match self.count.compare_exchange_weak(A::ONE, A::ZERO, Ordering::SeqCst, Ordering::SeqCst){
                    Ok(_) => return true,
                    Err(new_val) => prev_readers = new_val,
                }
            }
            else{
                return false;
            }
        }
    }

    pub fn try_read_static(self: &Arc<Self>) -> Option<AtomicArcReader<T, A>> {
        if self.lock_read(){
            Some(AtomicArcReader::new(self.clone()))
        }
        else{
            None
        }
    }

    pub fn try_write_static(self: &Arc<Self>) -> Option<AtomicArcWriter<T, A>> {
        if self.lock_write(){
            Some(AtomicArcWriter::new(self.clone()))
        }
        else{
            None
        }
    }

    pub fn try_read(&self) -> Option<AtomicRefReader<'_, T, A>>{
        if self.lock_read(){
            Some(AtomicRefReader::new(self))
        }
        else{
            None
        }
    }

    pub fn try_write(&self) -> Option<AtomicRefWriter<'_, T, A>>{
        if self.lock_write(){
            Some(AtomicRefWriter::new(self))
        }
        else{
            None
        }
    }
}
impl<T, A> Default for AtomicRwLock<T, A> where T: Default, A: AtomicValue, {
    fn default() -> Self {
        Self { count: A::new(A::ONE), data: UnsafeCell::default() }
    }
}
impl<T, A> From<T> for AtomicRwLock<T, A> where T: Default, A: AtomicValue, {
    fn from(from: T) -> Self {
        Self::new(from)
    }
}

#[derive(Debug)]
pub struct AtomicRwLockReader<T, A, L> where L: Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue {
    lock: L,
}
impl<T, A, L> AtomicRwLockReader<T, A, L> where L: Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue {
    fn new(lock: L) -> Self {
        Self { lock }
    }
}
impl<T, A, L> Deref for AtomicRwLockReader<T, A, L> where L: Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}
impl<T, A, L> Drop for AtomicRwLockReader<T, A, L> where L: Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue {
    fn drop(&mut self) {
        #[cfg(debug_assertions)] {
            let last = self.lock.count.fetch_sub(A::ONE, Ordering::SeqCst);
            if last <= A::ONE {
                panic!("Reader was dropped and lock count was less than or one!: {:?}", last)
            }
        }
        #[cfg(not(debug_assertions))] {
            self.lock.count.fetch_sub(A::ONE, Ordering::SeqCst);
        }
    }
}
impl<T, A, L> EnsureSend for AtomicRwLockReader<T, A, L> where L: Send + Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue {}
impl<T, A, L> EnsureSync for AtomicRwLockReader<T, A, L> where L: Sync + Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue {}

#[derive(Debug)]
pub struct AtomicRwLockWriter<T, A, L> where L: Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue {
    lock: Option<L>,
}
impl<T, A, L> AtomicRwLockWriter<T, A, L> where L: Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue{
    fn new(lock: L) -> Self{
        Self{ lock: Some(lock) }
    }

    pub fn into_inner(mut self) -> L{
        self.lock.take().unwrap()
    }
    pub fn downgrade(self) -> AtomicRwLockReader<T, A, L>{
        self.lock.as_ref().unwrap().count.fetch_add(A::ONE + A::ONE, Ordering::SeqCst);
        AtomicRwLockReader{ lock: self.into_inner() }
    }
}
impl<T, A, L> Deref for AtomicRwLockWriter<T, A, L> where L: Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe{ &*self.lock.as_ref().unwrap().data.get() }
    }
}
impl<T, A, L> DerefMut for AtomicRwLockWriter<T, A, L> where L: Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe{ &mut *self.lock.as_ref().unwrap().data.get() }
    }
}
impl<T, A, L> Drop for AtomicRwLockWriter<T, A, L>where L: Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue{
    fn drop(&mut self) {
        if let Some(lock) = &self.lock {
            #[cfg(debug_assertions)] {
                lock.count.compare_exchange(A::ZERO, A::ONE, Ordering::SeqCst, Ordering::SeqCst).expect("Writer was dropped and lock count was not 0!");
            }
            #[cfg(not(debug_assertions))] {
                lock.count.fetch_add(A::ONE, Ordering::SeqCst);
            }
        }
    }
}
impl<T, A, L> EnsureSend for AtomicRwLockWriter<T, A, L>where L: Send + Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue{}
impl<T, A, L> EnsureSync for AtomicRwLockWriter<T, A, L>where L: Sync + Deref<Target=AtomicRwLock<T, A>>, A: AtomicValue{}

/// This trait is unsafe because is must be implemented on an atomic.
/// If implementation is not correct this will create unsafe situations.
pub unsafe trait AtomicValue {
    type Stored: Copy + PartialOrd + Ord + PartialEq + Eq + Add<Output=Self::Stored> + Debug;
    const ZERO: Self::Stored;
    const ONE: Self::Stored;

    fn new(val: Self::Stored) -> Self where Self: Sized;
    fn load(&self, ordering: Ordering) -> Self::Stored;
    fn compare_exchange(&self, current: Self::Stored, new: Self::Stored, success: Ordering, failure: Ordering, ) -> Result<Self::Stored, Self::Stored>;
    fn compare_exchange_weak(&self, current: Self::Stored, new: Self::Stored, success: Ordering, failure: Ordering, ) -> Result<Self::Stored, Self::Stored>;
    fn fetch_add(&self, val: Self::Stored, ordering: Ordering) -> Self::Stored;
    fn fetch_sub(&self, val: Self::Stored, ordering: Ordering) -> Self::Stored;
}
macro_rules! impl_atomic_value {
    ($ident:ident, $stored:ty) => {
        unsafe impl AtomicValue for $ident {
            type Stored = $stored;
            const ZERO: Self::Stored = 0 as $stored;
            const ONE: Self::Stored = 1 as $stored;

            #[inline]
            fn new(val: Self::Stored) -> Self where Self: Sized{
                Self::new(val)
            }

            #[inline]
            fn load(&self, ordering: Ordering) -> Self::Stored {
                self.load(ordering)
            }

            #[inline]
            fn compare_exchange(&self, current: Self::Stored, new: Self::Stored, success: Ordering, failure: Ordering) -> Result<Self::Stored, Self::Stored> {
                self.compare_exchange(current, new, success, failure)
            }

            #[inline]
            fn compare_exchange_weak(&self, current: Self::Stored, new: Self::Stored, success: Ordering, failure: Ordering) -> Result<Self::Stored, Self::Stored> {
                self.compare_exchange_weak(current, new, success, failure)
            }

            #[inline]
            fn fetch_add(&self, val: Self::Stored, ordering: Ordering) -> Self::Stored{
                self.fetch_add(val, ordering)
            }

            #[inline]
            fn fetch_sub(&self, val: Self::Stored, ordering: Ordering) -> Self::Stored{
                self.fetch_sub(val, ordering)
            }
        }
    };
}
impl_atomic_value!(AtomicU8, u8);
impl_atomic_value!(AtomicU16, u16);
impl_atomic_value!(AtomicU32, u32);
impl_atomic_value!(AtomicU64, u64);
impl_atomic_value!(AtomicUsize, usize);

#[cfg(test)]
mod test{
    use crate::atomic_rw_lock::{AtomicRwLock, AtomicValue};
    use std::sync::atomic::Ordering;
    use std::ops::{DerefMut, Deref};

    fn single_thread_test<A: AtomicValue>(lock: AtomicRwLock<String, A>){
        assert_eq!(A::ONE, lock.count.load(Ordering::SeqCst));
        let mut write_guard = lock.try_write().expect("Could not lock when writer available");
        *write_guard.deref_mut() = "Test1".to_string();
        assert_eq!(A::ZERO, lock.count.load(Ordering::SeqCst));
        assert!(lock.try_write().is_none());
        assert!(lock.try_read().is_none());
        assert!(lock.try_write().is_none());
        assert!(lock.try_read().is_none());
        drop(write_guard);
        assert_eq!(A::ONE, lock.count.load(Ordering::SeqCst));
        let read_guard = lock.try_read().expect("Could not lock when reader available");
        assert_eq!(A::ONE + A::ONE, lock.count.load(Ordering::SeqCst));
        assert!(lock.try_write().is_none());
        assert!(lock.try_read().is_some());
        assert!(lock.try_write().is_none());
        assert!(lock.try_read().is_some());
        assert_eq!(read_guard.deref(), "Test1");
        drop(read_guard);
        assert_eq!(A::ONE, lock.count.load(Ordering::SeqCst));
        let mut write_guard = lock.try_write().expect("Could not lock when writer was available again");
        *write_guard.deref_mut() = "Test2".to_string();
        assert!(lock.try_write().is_none());
        assert!(lock.try_read().is_none());
        assert!(lock.try_write().is_none());
        assert!(lock.try_read().is_none());
        let read_guard = write_guard.downgrade();
        assert_eq!(A::ONE + A::ONE, lock.count.load(Ordering::SeqCst));
        assert!(lock.try_write().is_none());
        assert!(lock.try_read().is_some());
        assert!(lock.try_write().is_none());
        assert!(lock.try_read().is_some());
        assert_eq!(read_guard.deref(), "Test2");
        drop(read_guard);
        assert_eq!(A::ONE, lock.count.load(Ordering::SeqCst));
    }

    #[test]
    fn test_u8(){
        single_thread_test(AtomicRwLock::new_u8("".to_string()));
    }
    #[test]
    fn test_u16(){
        single_thread_test(AtomicRwLock::new_u16("".to_string()));
    }
    #[test]
    fn test_u32(){
        single_thread_test(AtomicRwLock::new_u32("".to_string()));
    }
    #[test]
    fn test_u64(){
        single_thread_test(AtomicRwLock::new_u64("".to_string()));
    }
    #[test]
    fn test_usize(){
        single_thread_test(AtomicRwLock::new_usize("".to_string()));
    }
}
