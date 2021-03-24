/// Prevents memory leaks, very unsafe unless you know that all copies of this reference are dropped before this.
/// This means that all returned references to this static should not last longer than this object as well as this hould be dropped last.
#[derive(Debug)]
pub struct StaticRefHolder<T>
where
    T: 'static,
{
    reference: *mut T,
}
impl<T> StaticRefHolder<T> {
    pub fn new(reference: &mut T) -> Self {
        Self { reference }
    }
}
impl<T> Drop for StaticRefHolder<T> {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.reference);
        }
    }
}
/// T is never accessed from this so if T can be sent StaticRefHolder<T> can be both Send and Sync
unsafe impl<T> Send for StaticRefHolder<T> where T: Send {}
/// T is never accessed from this so if T can be sent StaticRefHolder<T> can be both Send and Sync
unsafe impl<T> Sync for StaticRefHolder<T> where T: Send {}
