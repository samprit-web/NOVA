//! Collection types

use crate::prelude::*;
use core::hash::{Hash, Hasher, BuildHasher};
use core::fmt;
use core::cmp::Ordering;
use core::iter::{FromIterator, FusedIterator};
use core::borrow::Borrow;
use core::ops::{Index, IndexMut, RangeBounds};

// Vec
pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

struct RawVec<T> {
    ptr: *mut T,
    cap: usize,
}

impl<T> Vec<T> {
    pub const fn new() -> Self {
        Self { buf: RawVec::new(), len: 0 }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { buf: RawVec::with_capacity(capacity), len: 0 }
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.buf.cap {
            self.buf.grow();
        }
        unsafe {
            core::ptr::write(self.buf.ptr.add(self.len), value);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(core::ptr::read(self.buf.ptr.add(self.len))) }
        }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn capacity(&self) -> usize { self.buf.cap }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.buf.ptr.add(index)) }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.buf.ptr.add(index)) }
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.buf.ptr, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.buf.ptr, self.len) }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter { slice: self.as_slice(), index: 0 }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut { slice: self.as_mut_slice(), index: 0 }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> RawVec<T> {
    const fn new() -> Self {
        Self { ptr: core::ptr::null_mut(), cap: 0 }
    }

    fn with_capacity(cap: usize) -> Self {
        if cap == 0 {
            Self::new()
        } else {
            let layout = core::alloc::Layout::array::<T>(cap).unwrap();
            let ptr = unsafe { core::alloc::alloc(layout) as *mut T };
            if ptr.is_null() {
                core::alloc::handle_alloc_error(layout);
            }
            Self { ptr, cap }
        }
    }

    fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 4 } else { self.cap * 2 };
        let layout = core::alloc::Layout::array::<T>(new_cap).unwrap();
        let new_ptr = unsafe { core::alloc::realloc(self.ptr as *mut u8, layout, new_cap * core::mem::size_of::<T>()) };
        if new_ptr.is_null() {
            core::alloc::handle_alloc_error(layout);
        }
        self.ptr = new_ptr as *mut T;
        self.cap = new_cap;
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            let layout = core::alloc::Layout::array::<T>(self.cap).unwrap();
            unsafe { core::alloc::dealloc(self.ptr as *mut u8, layout) };
        }
    }
}

// Iterators
pub struct Iter<'a, T> {
    slice: &'a [T],
    index: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slice.len() {
            let item = &self.slice[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.slice.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}
impl<'a, T> FusedIterator for Iter<'a, T> {}

pub struct IterMut<'a, T> {
    slice: &'a mut [T],
    index: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slice.len() {
            let item = &mut self.slice[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

// HashMap
pub struct HashMap<K, V, S = RandomState> {
    map: hashbrown::HashMap<K, V, S>,
}

impl<K, V> HashMap<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self { map: hashbrown::HashMap::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { map: hashbrown::HashMap::with_capacity(capacity) }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }

    pub fn len(&self) -> usize { self.map.len() }
    pub fn is_empty(&self) -> bool { self.map.is_empty() }
    pub fn contains_key(&self, key: &K) -> bool { self.map.contains_key(key) }
}

// HashSet
pub struct HashSet<T, S = RandomState> {
    set: hashbrown::HashSet<T, S>,
}

impl<T> HashSet<T>
where
    T: Eq + Hash,
{
    pub fn new() -> Self {
        Self { set: hashbrown::HashSet::new() }
    }

    pub fn insert(&mut self, value: T) -> bool {
        self.set.insert(value)
    }

    pub fn contains(&self, value: &T) -> bool {
        self.set.contains(value)
    }

    pub fn remove(&mut self, value: &T) -> bool {
        self.set.remove(value)
    }

    pub fn len(&self) -> usize { self.set.len() }
    pub fn is_empty(&self) -> bool { self.set.is_empty() }
}

// BTreeMap
pub struct BTreeMap<K, V> {
    map: indexmap::BTreeMap<K, V>,
}

impl<K, V> BTreeMap<K, V>
where
    K: Ord,
{
    pub fn new() -> Self {
        Self { map: indexmap::BTreeMap::new() }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }

    pub fn len(&self) -> usize { self.map.len() }
    pub fn is_empty(&self) -> bool { self.map.is_empty() }
}

// VecDeque
pub struct VecDeque<T> {
    buf: RawVec<T>,
    head: usize,
    len: usize,
}

impl<T> VecDeque<T> {
    pub fn new() -> Self {
        Self { buf: RawVec::new(), head: 0, len: 0 }
    }

    pub fn push_front(&mut self, value: T) {
        if self.len == self.buf.cap {
            self.grow();
        }
        self.head = (self.head + self.buf.cap - 1) % self.buf.cap.max(1);
        unsafe { core::ptr::write(self.buf.ptr.add(self.head), value); }
        self.len += 1;
    }

    pub fn push_back(&mut self, value: T) {
        if self.len == self.buf.cap {
            self.grow();
        }
        let idx = (self.head + self.len) % self.buf.cap.max(1);
        unsafe { core::ptr::write(self.buf.ptr.add(idx), value); }
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 { return None; }
        let val = unsafe { core::ptr::read(self.buf.ptr.add(self.head)) };
        self.head = (self.head + 1) % self.buf.cap;
        self.len -= 1;
        Some(val)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 { return None; }
        let idx = (self.head + self.len - 1) % self.buf.cap;
        let val = unsafe { core::ptr::read(self.buf.ptr.add(idx)) };
        self.len -= 1;
        Some(val)
    }

    fn grow(&mut self) {
        let new_cap = (self.buf.cap.max(4) * 2).next_power_of_two();
        let layout = core::alloc::Layout::array::<T>(new_cap).unwrap();
        let new_ptr = unsafe { core::alloc::alloc(layout) as *mut T };
        if new_ptr.is_null() { core::alloc::handle_alloc_error(layout); }

        // Copy elements
        for i in 0..self.len {
            let src = (self.head + i) % self.buf.cap;
            unsafe { core::ptr::write(new_ptr.add(i), core::ptr::read(self.buf.ptr.add(src))); }
        }

        if self.buf.cap > 0 {
            let old_layout = core::alloc::Layout::array::<T>(self.buf.cap).unwrap();
            unsafe { core::alloc::dealloc(self.buf.ptr as *mut u8, old_layout); }
        }

        self.buf.ptr = new_ptr;
        self.buf.cap = new_cap;
        self.head = 0;
    }
}

// LinkedList
pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    tail: *mut Node<T>,
    len: usize,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self { head: None, tail: core::ptr::null_mut(), len: 0 }
    }

    pub fn push_front(&mut self, value: T) {
        let mut node = Box::new(Node { value, next: None });
        let node_ptr = node.as_mut() as *mut Node<T>;
        if self.head.is_none() {
            self.tail = node_ptr;
        }
        node.next = self.head.take();
        self.head = Some(node);
        self.len += 1;
    }

    pub fn push_back(&mut self, value: T) {
        let mut node = Box::new(Node { value, next: None });
        let node_ptr = node.as_mut() as *mut Node<T>;
        if self.tail.is_null() {
            self.head = Some(node);
            self.tail = node_ptr;
        } else {
            unsafe { (*self.tail).next = Some(node); }
            self.tail = node_ptr;
        }
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|mut node| {
            self.head = node.next.take();
            if self.head.is_none() {
                self.tail = core::ptr::null_mut();
            }
            self.len -= 1;
            node.value
        })
    }
}

// BinaryHeap
pub struct BinaryHeap<T> {
    data: Vec<T>,
}

impl<T> BinaryHeap<T>
where
    T: Ord,
{
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
        self.sift_up(self.data.len() - 1);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.data.is_empty() { return None; }
        let max = self.data.swap_remove(0);
        if !self.data.is_empty() {
            self.sift_down(0);
        }
        Some(max)
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.first()
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }

    fn sift_up(&mut self, mut idx: usize) {
        while idx > 0 {
            let parent = (idx - 1) / 2;
            if self.data[parent] >= self.data[idx] { break; }
            self.data.swap(parent, idx);
            idx = parent;
        }
    }

    fn sift_down(&mut self, mut idx: usize) {
        let len = self.data.len();
        loop {
            let left = 2 * idx + 1;
            let right = 2 * idx + 2;
            let mut largest = idx;

            if left < len && self.data[left] > self.data[largest] {
                largest = left;
            }
            if right < len && self.data[right] > self.data[largest] {
                largest = right;
            }
            if largest == idx { break; }

            self.data.swap(idx, largest);
            idx = largest;
        }
    }
}

// Re-export hashbrown types
pub use hashbrown::hash_map::Entry;
pub use hashbrown::hash_map::Keys;
pub use hashbrown::hash_map::Values;
pub use hashbrown::hash_map::Iter as HashMapIter;

// Vec macros
#[macro_export]
macro_rules! vec {
    () => { $crate::collections::Vec::new() };
    ($($x:expr),+ $(,)?) => {{
        let mut v = $crate::collections::Vec::with_capacity(0 $(+ 1)*);
        $(v.push($x);)+
        v
    }};
    ($elem:expr; $n:expr) => {{
        let mut v = $crate::collections::Vec::with_capacity($n);
        v.resize($n, $elem);
        v
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec() {
        let mut v = Vec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.len(), 3);
        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.pop(), Some(2));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn test_hashmap() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        m.insert("b", 2);
        assert_eq!(m.get(&"a"), Some(&1));
        assert_eq!(m.get(&"b"), Some(&2));
        assert_eq!(m.get(&"c"), None);
    }

    #[test]
    fn test_hashset() {
        let mut s = HashSet::new();
        s.insert(1);
        s.insert(2);
        assert!(s.contains(&1));
        assert!(s.contains(&2));
        assert!(!s.contains(&3));
    }

    #[test]
    fn test_vecdeque() {
        let mut d = VecDeque::new();
        d.push_front(1);
        d.push_back(2);
        d.push_front(0);
        assert_eq!(d.pop_front(), Some(0));
        assert_eq!(d.pop_front(), Some(1));
        assert_eq!(d.pop_back(), Some(2));
        assert_eq!(d.pop_front(), None);
    }

    #[test]
    fn test_binary_heap() {
        let mut h = BinaryHeap::new();
        h.push(1);
        h.push(3);
        h.push(2);
        assert_eq!(h.pop(), Some(3));
        assert_eq!(h.pop(), Some(2));
        assert_eq!(h.pop(), Some(1));
    }
}