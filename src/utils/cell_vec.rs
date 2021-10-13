use core::ptr;
use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use std::mem::MaybeUninit;

/// dynamic fixed length vector with shared mutation via `Cell`
pub struct CellVec<T> {
    storage: Box<[Cell<MaybeUninit<T>>]>,
    length: Cell<usize>,
}
// invariant that must be uphold
// storage[i] is initialized, for every 0 <= i < length
// shared reference do not give out reference to user (but exclusive reference can)

impl<T> CellVec<T> {
    // TODO: abstract & get_unchecked

    pub fn new(capacity: usize) -> Self {
        let mut storage = Vec::with_capacity(capacity);
        storage.extend(std::iter::repeat_with(|| Cell::new(MaybeUninit::uninit())).take(capacity));
        CellVec {
            storage: storage.into_boxed_slice(),
            length: Cell::new(0),
        }
    }

    pub fn reallocate() {
        unimplemented!()
    }

    pub fn last_index(&self) -> Option<usize> {
        self.length.get().checked_sub(1)
    }

    pub fn len(&self) -> usize {
        self.length.get()
    }

    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Return slice of all initialized cell.
    ///
    /// May cause UB if mutation (including interior mutation) occurred
    /// while still holding on to this slice
    pub unsafe fn _as_cell_slice<'a>(&'a self) -> &'a [Cell<T>] {
        // safe because
        // start[..self.length] is properly initialized
        // self.length isn't > storage capacity
        // Cell<MaybeUninit<T>> can be cast to Cell<T> (both Cell and MaybeUninit are #[repr(transparent)])
        let start: *const Cell<T> = self.storage.as_ptr().cast();
        std::slice::from_raw_parts::<'a, Cell<T>>(start, self.length.get())
    }

    /// safe version of `_as_cell_slice`, safety come from the use of exclusive reference
    pub fn as_cell_slice(&mut self) -> &[Cell<T>] {
        // UB won't occurred since user cannot hold the cell slice while mutating due to
        // the nature of exclusive reference
        unsafe { self._as_cell_slice() }
    }

    pub fn push(&self, value: T) -> Result<(), ()> {
        let len = self.length.get();
        self.storage
            .get(len)
            .ok_or(())?
            .set(MaybeUninit::new(value));
        self.length.set(len + 1);
        Ok(())
    }

    pub unsafe fn push_unchecked(&self, value: T) {
        let len = self.length.get();
        self.storage.get_unchecked(len).set(MaybeUninit::new(value));
        self.length.set(len + 1);
    }

    pub fn clear(&self) {
        self.truncate(0);
    }

    pub fn truncate(&self, len: usize) {
        let mut length = self.length.get();
        while len < length {
            // decrement len before the drop_in_place(), so a panic on Drop
            // doesn't re-drop the just-failed value.
            length -= 1;
            self.length.set(length);

            let cell = &self.storage[length];
            unsafe { uninitialize_cell(cell) };
        }
    }
}

impl<T: Copy> CellVec<T> {
    pub fn pop(&self) -> Option<T> {
        let val = {
            // safe because cell slice is dropped before mutation (length decrease)
            unsafe { self._as_cell_slice() }.last()?.get()
        };

        // Note: no dropping is need because of copy type

        // will always be valid length,
        // because storage won't be empty since cell_slice didn't early return
        self.length.set(self.length.get() - 1);
        Some(val)
    }

    /// fast clearing by not dropping. so it only available on copy type
    pub fn clear_fast(&self) {
        self.length.set(0);
    }

    pub fn as_slice<'a>(&'a mut self) -> &'a [T] {
        // safe because
        // T is copy, so it's in line with cell's get semantic
        // exclusive reference, so user cannot holding on to thee return slice for too long
        // data from start to start+length is properly initialize

        let start: *const T = self.storage.as_ptr().cast();
        unsafe { std::slice::from_raw_parts::<'a, T>(start, self.length.get()) }
    }

    pub fn get(&self, index: usize) -> Option<T> {
        // safe because there is no interior mutation
        unsafe { self._as_cell_slice() }
            .get(index)
            .map(|cell| cell.get())
    }

    /// panic version of `get`.
    // cannot use Index trait because it cannot return a value, only reference
    pub fn index(&self, index: usize) -> T {
        self.get(index).expect("index out of bounded")
    }

    /// unchecked version of `get`, safe when index < length (like normal slice)
    pub unsafe fn get_unchecked(&self, index: usize) -> T {
        // safe because there is no interior mutation
        unsafe { self._as_cell_slice() }.get_unchecked(index).get()
    }
}

impl<T> Drop for CellVec<T> {
    fn drop(&mut self) {
        // drop initialized member
        // maybe uninit won't auto drop themself
        for i in self.as_cell_slice() {
            unsafe { ptr::drop_in_place(i.as_ptr()) };
        }
    }
}

impl<T: Debug + Copy> Debug for CellVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        write!(f, "\tdata: ")?;
        // safe because there is no interior mutation
        if let Some((last, rest)) = unsafe { self._as_cell_slice() }.split_last() {
            write!(f, "[")?;
            rest.iter().try_for_each(|cell| {
                write!(f, "{:?}, ", cell.get())?;
                Ok(())
            })?;
            write!(f, "{:?}]", last.get())?;
        } else {
            write!(f, "\t[]")?;
        }
        writeln!(f, "\n\tcapacity: {}", self.storage.len())?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

/// Drop content inside MaybeUninit Cell, leaving it uninitialized.
/// Content must be valid & initialized
unsafe fn uninitialize_cell<T>(x: &Cell<MaybeUninit<T>>) {
    let temp_cell = Cell::new(MaybeUninit::uninit());
    x.swap(&temp_cell);
    drop(temp_cell.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let v = CellVec::<usize>::new(0);
        dbg!(&v);
        assert_eq!(v.len(), 0);
        assert_eq!(v.capacity(), 0);

        let v = CellVec::<usize>::new(10);
        dbg!(&v);
        assert_eq!(v.len(), 0);
        assert_eq!(v.capacity(), 10);
    }

    #[test]
    fn push_n_pop() {
        let v = CellVec::new(3);
        v.push("1a").expect("able to push");
        v.push("2b").expect("able to push");
        v.push("3c").expect("able to push");
        v.push("4b")
            .expect_err("since capacity full, it should be unable to push");
        v.push("5b")
            .expect_err("since capacity full, it should be unable to push");

        dbg!(&v);

        assert_eq!(v.pop(), Some("3c"));
        assert_eq!(v.pop(), Some("2b"));
        assert_eq!(v.pop(), Some("1a"));
        assert_eq!(v.pop(), None);
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn clear() {
        let v = CellVec::<&'static str>::new(10);
        v.push("1").expect("able to push");
        v.push("1").expect("able to push");
        v.push("1").expect("able to push");

        v.clear();
    }
}
