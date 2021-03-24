use crate::rw_lock_cycler::{RwLockCyclerWriter, RwLockCyclerReader, RwLockCycler};
use std::sync::atomic::AtomicU8;

#[cfg(feature = "unsafe_cleanup")]
use std::sync::Arc;
#[cfg(feature = "unsafe_cleanup")]
use crate::static_ref_holder::StaticRefHolder;
use parking_lot::RwLock;

/// Creates a single reader RwLockCycler using `values` as the initial values for the slots.
pub fn build_single_reader<T>(values: [T; 3]) -> (RwLockCyclerWriter<T>, RwLockCyclerReader<T>) {
    let [a, b, c] = values;
    let cycler = Box::leak(Box::new(
        RwLockCycler {
            data_slots: Box::new([RwLock::new(a), RwLock::new(b), RwLock::new(c)]) as Box<[RwLock<T>]>,
            most_up_to_date: AtomicU8::new(0),
        }
    ));
    #[cfg(feature = "unsafe_cleanup")]
        let ref_holder = Arc::new(StaticRefHolder::new(cycler));
    (
        RwLockCyclerWriter {
            #[cfg(feature = "unsafe_cleanup")]
            ref_holder: ref_holder.clone(),
            cycler,
            writer: cycler.data_slots[1].write(),
            currently_writing: 1,
        },
        RwLockCyclerReader {
            #[cfg(feature = "unsafe_cleanup")]
            ref_holder,
            cycler,
            reader: Some(cycler.data_slots[0].read()),
        }
    )
}

/// Creates a multi reader RwLockCycler, the amount of readers being `initial_values.len() - 2`.
pub fn build_multiple_reader<T>(initial_values: Vec<T>) -> (RwLockCyclerWriter<T>, Vec<RwLockCyclerReader<T>>) {
    #[cfg(debug_assertions)]
    assert!(initial_values.len() >= 3 && initial_values.len() <= u8::MAX as usize);
    let cycler = Box::leak(Box::new(
        RwLockCycler {
            data_slots: initial_values.into_iter().map(RwLock::new).collect(),
            most_up_to_date: AtomicU8::new(0),
        }
    ));
    #[cfg(feature = "unsafe_cleanup")]
        let ref_holder = Arc::new(StaticRefHolder::new(cycler));
    let mut readers = Vec::with_capacity(cycler.num_readers());
    for _ in 0..cycler.num_readers() {
        readers.push(RwLockCyclerReader {
            #[cfg(feature = "unsafe_cleanup")]
            ref_holder: ref_holder.clone(),
            reader: Some(cycler.data_slots[0].read()),
            cycler,
        })
    }
    (
        RwLockCyclerWriter {
            #[cfg(feature = "unsafe_cleanup")]
            ref_holder,
            writer: cycler.data_slots[1].write(),
            cycler,
            currently_writing: 1,
        },
        readers,
    )
}
