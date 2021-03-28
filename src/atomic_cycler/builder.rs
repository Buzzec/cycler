use crate::atomic_cycler::writer::AtomicCyclerWriter;
use crate::atomic_cycler::reader::AtomicCyclerReader;
use crate::atomic_rw_lock::AtomicRwLock;
use std::sync::atomic::AtomicU8;
use crate::atomic_cycler::AtomicCycler;
use std::sync::Arc;
/// Creates a single reader RwLockCycler using `values` as the initial values for the slots.
pub fn build_single_reader<T>(values: [T; 3]) -> (AtomicCyclerWriter<T>, AtomicCyclerReader<T>) {
    let [a, b, c] = values;
    let cycler = Arc::new(AtomicCycler {
        data_slots: Box::new([Arc::new(AtomicRwLock::new_u8(a)), Arc::new(AtomicRwLock::new_u8(b)), Arc::new(AtomicRwLock::new_u8(c))]) as Box<[Arc<AtomicRwLock<T, AtomicU8>>]>,
        most_up_to_date: AtomicU8::new(0),
    });
    (
        AtomicCyclerWriter {
            cycler: cycler.clone(),
            writer: cycler.data_slots[1].try_write_static().unwrap(),
            currently_writing: 1,
        },
        AtomicCyclerReader {
            reader: Some(cycler.data_slots[0].try_read_static().unwrap()),
            cycler,
        },
    )
}

/// Creates a multi reader RwLockCycler, the amount of readers being `initial_values.len() - 2`.
pub fn build_multiple_reader<T>(initial_values: Vec<T>) -> (AtomicCyclerWriter<T>, Vec<AtomicCyclerReader<T>>) {
    #[cfg(debug_assertions)]
    assert!(initial_values.len() >= 3 && initial_values.len() <= u8::MAX as usize);
    let cycler = Arc::new(AtomicCycler {
        data_slots: initial_values.into_iter().map(|val|Arc::new(AtomicRwLock::new_u8(val))).collect(),
        most_up_to_date: AtomicU8::new(0),
    });
    let mut readers = Vec::with_capacity(cycler.num_readers());
    for _ in 0..cycler.num_readers() {
        readers.push(AtomicCyclerReader {
            reader: Some(cycler.data_slots[0].try_read_static().unwrap()),
            cycler: cycler.clone(),
        })
    }
    (
        AtomicCyclerWriter {
            writer: cycler.data_slots[1].try_write_static().unwrap(),
            cycler,
            currently_writing: 1,
        },
        readers,
    )
}
