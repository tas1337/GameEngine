// Custom memory management - built from scratch!

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

/// Arena allocator for fast, bulk allocations
pub struct Arena {
    buffer: NonNull<u8>,
    capacity: usize,
    offset: usize,
}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 16).expect("Invalid layout");
        let buffer = unsafe {
            let ptr = alloc(layout);
            NonNull::new(ptr).expect("Allocation failed")
        };

        Self {
            buffer,
            capacity,
            offset: 0,
        }
    }

    /// Allocate memory from the arena
    pub fn alloc<T>(&mut self, count: usize) -> Option<NonNull<T>> {
        let size = std::mem::size_of::<T>() * count;
        let align = std::mem::align_of::<T>();

        // Align offset
        let padding = (align - (self.offset % align)) % align;
        let aligned_offset = self.offset + padding;

        if aligned_offset + size > self.capacity {
            return None; // Out of memory
        }

        let ptr = unsafe {
            let ptr = self.buffer.as_ptr().add(aligned_offset) as *mut T;
            NonNull::new_unchecked(ptr)
        };

        self.offset = aligned_offset + size;
        Some(ptr)
    }

    /// Reset the arena (doesn't free memory, just resets offset)
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    pub fn used(&self) -> usize {
        self.offset
    }

    pub fn available(&self) -> usize {
        self.capacity - self.offset
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(self.capacity, 16);
            dealloc(self.buffer.as_ptr(), layout);
        }
    }
}

/// Pool allocator for fixed-size objects
pub struct Pool<T> {
    slots: Vec<Option<T>>,
    free_list: Vec<usize>,
}

impl<T> Pool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        let mut free_list = Vec::with_capacity(capacity);

        for i in 0..capacity {
            slots.push(None);
            free_list.push(capacity - i - 1); // Reverse order for pop efficiency
        }

        Self { slots, free_list }
    }

    /// Allocate an object from the pool
    pub fn alloc(&mut self, value: T) -> Option<usize> {
        if let Some(index) = self.free_list.pop() {
            self.slots[index] = Some(value);
            Some(index)
        } else {
            None // Pool is full
        }
    }

    /// Free an object back to the pool
    pub fn free(&mut self, index: usize) {
        if index < self.slots.len() && self.slots[index].is_some() {
            self.slots[index] = None;
            self.free_list.push(index);
        }
    }

    /// Get a reference to an object
    pub fn get(&self, index: usize) -> Option<&T> {
        self.slots.get(index).and_then(|slot| slot.as_ref())
    }

    /// Get a mutable reference to an object
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.slots.get_mut(index).and_then(|slot| slot.as_mut())
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    pub fn used(&self) -> usize {
        self.slots.len() - self.free_list.len()
    }

    /// Iterate over all allocated objects
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.as_ref().map(|v| (i, v)))
    }

    /// Iterate mutably over all allocated objects
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(i, slot)| slot.as_mut().map(|v| (i, v)))
    }
}

/// Handle for referencing pool objects safely
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle {
    pub index: usize,
    pub generation: u32,
}

impl Handle {
    pub const INVALID: Handle = Handle { index: usize::MAX, generation: 0 };

    pub fn new(index: usize, generation: u32) -> Self {
        Self { index, generation }
    }

    pub fn is_valid(&self) -> bool {
        *self != Self::INVALID
    }
}

/// Generational pool - handles are validated against generations
pub struct GenerationalPool<T> {
    slots: Vec<PoolSlot<T>>,
    free_list: Vec<usize>,
}

struct PoolSlot<T> {
    value: Option<T>,
    generation: u32,
}

impl<T> GenerationalPool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        let mut free_list = Vec::with_capacity(capacity);

        for i in 0..capacity {
            slots.push(PoolSlot {
                value: None,
                generation: 0,
            });
            free_list.push(capacity - i - 1);
        }

        Self { slots, free_list }
    }

    pub fn alloc(&mut self, value: T) -> Option<Handle> {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index];
            slot.value = Some(value);
            Some(Handle::new(index, slot.generation))
        } else {
            None
        }
    }

    pub fn free(&mut self, handle: Handle) -> bool {
        if let Some(slot) = self.slots.get_mut(handle.index) {
            if slot.generation == handle.generation && slot.value.is_some() {
                slot.value = None;
                slot.generation = slot.generation.wrapping_add(1);
                self.free_list.push(handle.index);
                return true;
            }
        }
        false
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        self.slots
            .get(handle.index)
            .and_then(|slot| {
                if slot.generation == handle.generation {
                    slot.value.as_ref()
                } else {
                    None
                }
            })
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        self.slots
            .get_mut(handle.index)
            .and_then(|slot| {
                if slot.generation == handle.generation {
                    slot.value.as_mut()
                } else {
                    None
                }
            })
    }
}

