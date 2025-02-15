//! Taken with modifications from std::collections::BinaryHeap.

pub(crate) fn sift_up<T: Ord>(v: &mut [T], start: usize, pos: usize) -> usize {
    // Take out the value at `pos` and create a hole.
    let mut hole = Hole::new(v, pos);

    while hole.pos() > start {
        let parent = (hole.pos() - 1) / 2;

        if hole.element() <= hole.get(parent) {
            break;
        }

        hole.move_to(parent)
    }

    hole.pos()
}

/// Take an element at `pos` and move it down the heap,
/// while its children are larger.
pub(crate) fn sift_down_range<T: Ord>(v: &mut [T], pos: usize, end: usize) {
    let mut hole = Hole::new(v, pos);
    let mut child = 2 * hole.pos() + 1;

    // Loop invariant: child == 2 * hole.pos() + 1.
    while child <= end.saturating_sub(2) {
        // compare with the greater of the two children
        child += (hole.get(child) <= hole.get(child + 1)) as usize;

        // if we are already in order, stop.
        if hole.element() >= hole.get(child) {
            return;
        }

        hole.move_to(child);
        child = 2 * hole.pos() + 1;
    }

    if child == end - 1 && hole.element() < hole.get(child) {
        hole.move_to(child);
    }
}

pub(crate) fn sift_down<T: Ord>(v: &mut [T], pos: usize) {
    let len = v.len();
    sift_down_range(v, pos, len);
}

pub(crate) fn rebuild<T: Ord>(v: &mut [T]) {
    let mut n = v.len() / 2;
    while n > 0 {
        n -= 1;
        sift_down(v, n);
    }
}

/// Hole represents a hole in a slice i.e., an index without valid value
/// (because it was moved from or duplicated).
/// In drop, `Hole` will restore the slice by filling the hole
/// position with the value that was originally removed.
struct Hole<'a, T: 'a> {
    data: &'a mut [T],
    pos: usize,
}

impl<'a, T> Hole<'a, T> {
    /// Creates a new `Hole` at index `pos`.
    ///
    /// Unsafe because pos must be within the data slice.
    #[inline]
    fn new(data: &'a mut [T], pos: usize) -> Self {
        Hole { data, pos }
    }

    #[inline]
    fn pos(&self) -> usize {
        self.pos
    }

    /// Returns a reference to the element removed.
    #[inline]
    fn element(&self) -> &T {
        self.get(self.pos)
    }

    /// Returns a reference to the element at `index`.
    #[inline]
    fn get(&self, index: usize) -> &T {
        self.data.get(index).unwrap()
    }

    /// Move hole to new location
    #[inline]
    fn move_to(&mut self, index: usize) {
        self.data.swap(index, self.pos);
        self.pos = index;
    }
}
