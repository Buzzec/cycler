macro_rules! rw_cycler_fn {
    ($self:ident, $clone_fn:ident) => {
        use std::ops::{Deref, DerefMut};
        let mut next_write = ($self.currently_writing + 1) % $self.cycler.data_slots.len() as u8;
        loop {
            if let Some(mut writer) =
                parking_lot::RwLock::try_write(&$self.cycler.data_slots[next_write as usize])
            {
                std::mem::swap(&mut $self.writer, &mut writer);
                let old = parking_lot::RwLockWriteGuard::downgrade(writer);
                $clone_fn($self.writer.deref_mut(), old.deref());
                $self
                    .cycler
                    .most_up_to_date
                    .store($self.currently_writing, std::sync::atomic::Ordering::SeqCst);
                $self.currently_writing = next_write;
                return;
            } else {
                next_write = (next_write + 1) % $self.cycler.data_slots.len() as u8;
            }
        }
    };
}

macro_rules! rw_cycler_mut_fn {
    ($self:ident, $clone_fn:ident) => {
        use std::ops::DerefMut;
        let mut next_write = ($self.currently_writing + 1) % $self.cycler.data_slots.len() as u8;
        loop {
            if let Some(mut writer) =
                parking_lot::RwLock::try_write(&$self.cycler.data_slots[next_write as usize])
            {
                std::mem::swap(&mut $self.writer, &mut writer);
                $clone_fn($self.writer.deref_mut(), writer.deref_mut());
                $self
                    .cycler
                    .most_up_to_date
                    .store($self.currently_writing, std::sync::atomic::Ordering::SeqCst);
                $self.currently_writing = next_write;
                return;
            } else {
                next_write = (next_write + 1) % $self.cycler.data_slots.len() as u8;
            }
        }
    };
}
