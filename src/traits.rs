//! These are traits that are implemented for the types of cyclers.
//! `CyclerWriter` has many sub-traits that contain the functions allowing the writer to move to the next block,
//! while `CyclerReader` is a single trait that allows the reader to move to the latest block.
//! `WriteAccess` and `ReadAccess` are traits that should be implemented for any type that goes into a cycler.

/// Trait that can be implemented to ensure a type is send
pub trait EnsureSend: Send {}
/// Trait that can be implemented to ensure a type is sync
pub trait EnsureSync: Sync {}

/// This trait should be implemented for any type that can be written to by a cycler.
/// This specifically allows for the separation of read and write data from the cycler readers and writers.
/// If this functionality is not needed then `WriteAccess::Write` can be set to `Self` for no runtime cost.
pub trait WriteAccess {
    /// The type of write data that can be accessed.
    /// Will default to `Self` when https://github.com/rust-lang/rust/issues/29661 is resolved.
    type Write;

    /// Gets shared access to the write data contained
    fn write_data(&self) -> &Self::Write;
    /// Gets exclusive access to the write data contained
    fn write_data_mut(&mut self) -> &mut Self::Write;
}
/// Ensure `WriteAccess` can be trait object
impl<W> dyn WriteAccess<Write = W> {}

/// This trait should be implemented for any type that can be read from by a cycler.
/// This specifically allows for the separation of read and write data from the cycler readers and writers.
/// If this functionality is not needed then `ReadAccess::Read` can be set to `Self` for no runtime cost.
pub trait ReadAccess {
    /// The type of read data that can be accessed.
    /// Will default to `Self` when https://github.com/rust-lang/rust/issues/29661 is resolved.
    type Read;

    /// Gets shared access to the read data contained
    fn read_data(&self) -> &Self::Read;
}
/// Ensure `ReadAccess` can be trait object
impl<R> dyn ReadAccess<Read = R> {}

/// This trait is implemented for the write half of a cycler.
pub trait CyclerWriter<T>: WriteAccess<Write = T::Write>
where
    T: WriteAccess,
{
}
/// Ensure `CyclerWriter` can be trait object
impl<T> dyn CyclerWriter<T> where T: WriteAccess {}

/// This trait enables the write half of the cycler to be moved to the next chunk by a default copy function.
/// Usually this function is `Clone::clone_from` but that is not a strict requirement.
pub trait CyclerWriterDefault<T>: CyclerWriter<T>
where
    T: WriteAccess,
{
    /// Moves the writer to the next block cloning the previously written block using a default function.
    fn write_next(&mut self);
}
/// Ensure `CyclerWriterDefault` can be trait object
impl<T> dyn CyclerWriterDefault<T> where T: WriteAccess {}

/// This trait enables the write half of the cycler to move to the next block using a given clone function.
/// This function follows the signature of `Clone::clone_from`, meaning the arguments are (to, from).
pub trait CyclerWriterFn<T>: CyclerWriter<T>
where
    T: WriteAccess,
{
    /// Moves the writer to the next block cloning using an `fn` pointer.
    /// Arguments are (to, from)
    /// This function uses an `fn` pointer so has no additional runtime cost and can be called on a trait object.
    fn write_next_fn(&mut self, clone_fn: fn(&mut T, &T));
    /// Moves the writer to the next block cloning using an `FnOnce` impl
    /// Arguments are (to, from)
    /// This function is generic over the function reducing runtime cost but cannot be called on trait objects.
    fn write_next_fn_impl(&mut self, clone_fn: impl FnOnce(&mut T, &T))
    where
        Self: Sized;
    /// Moves the writer to the next block cloning using an `FnMut` dynamic reference.
    /// Arguments are (to, from)
    /// This function takes a dyn pointer so a v-table lookup is necessary.
    /// `write_next_fn` is preferred if the function can be an `fn` pointer or `write_next_fn_impl` is preferred if self is sized.
    fn write_next_fn_dyn(&mut self, clone_fn: &mut dyn FnMut(&mut T, &T));
    /// Moves the writer to the next block cloning using a boxed `FnOnce`
    /// Arguments are (to, from)
    /// This function takes a boxed dyn pointer so a v-table lookup is necessary.
    /// `write_next_fn_dyn` is preferred if the function can be coerced into an `FnMut` as no heap allocation will be necessary.
    fn write_next_fn_dyn_boxed(&mut self, clone_fn: Box<dyn FnOnce(&mut T, &T)>);
}
/// Ensure `CyclerWriterFn` can be trait object
impl<T> dyn CyclerWriterFn<T> where T: WriteAccess {}

/// This trait enables the write half of the cycler to move to the next block using a clone function that takes a mutable reference to the previous block.
/// This is not preferable as optimizations where the reader can read the previous block while it's being cloned are not possible.
/// The function arguments are (from, to)
pub trait CyclerWriterMutFn<T>: CyclerWriter<T>
where
    T: WriteAccess,
{
    /// Moves the writer to the next block cloning using an `fn` pointer.
    /// Arguments are (to, from)
    /// This function uses an `fn` pointer so has no additional runtime cost and can be called on a trait object.
    fn write_next_mut_fn(&mut self, clone_fn: fn(&mut T, &mut T));
    /// Moves the writer to the next block cloning using an `FnOnce` impl
    /// Arguments are (to, from)
    /// This function is generic over the function reducing runtime cost but cannot be called on trait objects.
    fn write_next_mut_fn_impl(&mut self, clone_fn: impl FnOnce(&mut T, &mut T))
    where
        Self: Sized;
    /// Moves the writer to the next block cloning using an `FnMut` dynamic reference.
    /// Arguments are (to, from)
    /// This function takes a dyn pointer so a v-table lookup is necessary.
    /// `write_next_mut_fn` is preferred if the function can be an `fn` pointer or `write_next_mut_fn_impl` is preferred if self is sized.
    fn write_next_mut_fn_dyn(&mut self, clone_fn: &mut dyn FnMut(&mut T, &mut T));
    /// Moves the writer to the next block cloning using a boxed `FnOnce`
    /// Arguments are (to, from)
    /// This function takes a boxed dyn pointer so a v-table lookup is necessary.
    /// `write_next_mut_fn_dyn` is preferred if the function can be coerced into an `FnMut` as no heap allocation will be necessary.
    fn write_next_mut_fn_dyn_boxed(&mut self, clone_fn: Box<dyn FnOnce(&mut T, &mut T)>);
}
/// Ensure `CyclerWriterMutClone` can be trait object
impl<T> dyn CyclerWriterMutFn<T> where T: WriteAccess {}

/// This trait is a collection of all the primarily supported writer traits.
/// Other traits may be added to this in the future but none will be taken away without a major version bump.
/// Other traits may also be added that do not fall under this for more specific functionality (ex: `CyclerWriterMutFn`).
pub trait UniversalCyclerWriter<T>: CyclerWriterFn<T> + CyclerWriterDefault<T>
where
    T: WriteAccess + Clone,
{
}
/// Ensure `UniversalCyclerWriter` can be trait object
impl<T> dyn UniversalCyclerWriter<T> where T: WriteAccess {}

/// This trait is implemented for the read half of a cycler.
pub trait CyclerReader<T>: ReadAccess<Read = T::Read>
where
    T: ReadAccess,
{
    /// Moves the reader to the most up-to-date block at the time of call.
    /// This may be the same block as previously read which means the writer has not published a new block in the time since the last call.
    fn read_latest(&mut self);
}
/// Ensure `CyclerReader` can be trait object
impl<T> dyn CyclerReader<T> where T: ReadAccess {}

/// This trait is a collection of all the primarily supported reader traits.
/// Other traits may be added to this in the future but none will be taken away without a major version bump.
/// Other traits may also be added that do not fall under this for more specific functionality.
pub trait UniversalCyclerReader<T>:
    CyclerWriterFn<T> + CyclerWriterMutFn<T> + CyclerWriterDefault<T>
where
    T: WriteAccess,
{
}
/// Ensure `UniversalCyclerReader` can be trait object
impl<T> dyn UniversalCyclerReader<T> where T: WriteAccess {}
