// Copyright 2017 Kyle Mayes
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Hazard pointers.

#![warn(missing_copy_implementations, missing_debug_implementations, missing_docs)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(clippy))]

use std::fmt;
use std::mem;
use std::ops;
use std::ptr;
use std::cell::{RefCell};
use std::sync::atomic::{AtomicPtr};
use std::sync::atomic::Ordering::*;

//================================================
// Traits
//================================================

// Memory ________________________________________

/// A type that can allocate and deallocate memory.
pub trait Memory {
    /// Allocates memory.
    unsafe fn allocate<T>(&self, value: T) -> *mut T;

    /// Deallocates the memory associated with the supplied pointer.
    unsafe fn deallocate<T>(&self, pointer: *mut T);
}

//================================================
// Structs
//================================================

// AlignVec ______________________________________

#[cfg(target_pointer_width="32")]
const POINTERS: usize = 32;
#[cfg(target_pointer_width="64")]
const POINTERS: usize = 16;

/// A `Vec` aligned to the size of a cacheline.
#[repr(C)]
pub struct AlignVec<T> {
    vec: Vec<T>,
    _padding: [usize; POINTERS - 3],
}

impl<T> AlignVec<T> {
    //- Constructors -----------------------------

    /// Constructs a new `AlignVec`.
    pub fn new(vec: Vec<T>) -> Self {
        AlignVec { vec: vec, _padding: [0; POINTERS - 3] }
    }
}

impl<T> fmt::Debug for AlignVec<T> where T: fmt::Debug {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", &self.vec)
    }
}

impl<T> ops::Deref for AlignVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T> ops::DerefMut for AlignVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

// Hazard ________________________________________

/// A collection of pointers considered hazardous.
#[repr(C)]
pub struct Hazard<T, M> where M: Memory {
    hazardous: AlignVec<Vec<AtomicPtr<T>>>,
    retired: AlignVec<RefCell<Vec<*mut T>>>,
    threshold: usize,
    memory: M,
}

impl<T, M> Hazard<T, M> where M: Memory {
    //- Constructors -----------------------------

    /// Constructs a new `Hazard`.
    ///
    /// The maximum number of threads is specified by `threads` and the maximum number of hazardous
    /// pointers per thread is specified by `domains`.
    ///
    /// The maximum size lists of retired pointers can grow to is specified by `threshold`. Once a
    /// list of retired pointers reaches this limit, any pointers that are no longer hazardous are
    /// removed from the list and the memory they refer to is deallocated.
    pub fn new(memory: M, threads: usize, domains: usize, threshold: usize) -> Self {
        let hazardous = (0..threads).map(|_| {
            (0..domains).map(|_| AtomicPtr::new(ptr::null_mut())).collect()
        }).collect();
        let retired = vec![RefCell::new(vec![]); threads];
        Hazard {
            hazardous: AlignVec::new(hazardous),
            retired: AlignVec::new(retired),
            threshold: threshold,
            memory: memory,
        }
    }

    //- Accessors --------------------------------

    /// Sets the hazardous pointer for the supplied domain using the supplied thread.
    pub fn mark(&self, thread: usize, domain: usize, pointer: *mut T) -> *mut T {
        self.hazardous[thread][domain].store(pointer, Release);
        pointer
    }

    /// Clears the hazardous pointer for the supplied domain using the supplied thread.
    pub fn clear(&self, thread: usize, domain: usize) {
        self.hazardous[thread][domain].store(ptr::null_mut(), Release);
    }

    /// Returns whether the supplied pointer is considered hazardous.
    pub fn hazardous(&self, pointer: *mut T) -> bool {
        self.hazardous.iter().any(|h| h.iter().any(|p| pointer == p.load(Acquire)))
    }

    fn kill(&self, pointer: *mut T) -> bool {
        if self.hazardous(pointer) {
            false
        } else {
            unsafe { self.memory.deallocate(pointer); }
            true
        }
    }

    /// Retires the supplied pointer using the supplied thread.
    pub fn retire(&self, thread: usize, pointer: *mut T) {
        let mut retired = self.retired[thread].borrow_mut();
        retired.push(pointer);
        if retired.len() >= self.threshold {
            retired.retain(|p| !self.kill(*p));
        }
    }
}

impl<T, M> Drop for Hazard<T, M> where M: Memory {
    fn drop(&mut self) {
        for retired in &*self.retired {
            for pointer in &*retired.borrow() {
                unsafe { self.memory.deallocate(*pointer); }
            }
        }
    }
}

impl<T, M> fmt::Debug for Hazard<T, M> where M: Memory {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Hazard").field("hazardous", &self.hazardous).finish()
    }
}

// VecMemory _____________________________________

/// An allocator that uses `Vec` to allocate and deallocate memory.
#[derive(Copy, Clone, Debug)]
pub struct VecMemory;

impl Memory for VecMemory {
    unsafe fn allocate<T>(&self, value: T) -> *mut T {
        let mut vec = vec![value];
        let pointer = vec.as_mut_ptr();
        mem::forget(vec);
        pointer
    }

    unsafe fn deallocate<T>(&self, pointer: *mut T) {
        assert!(!pointer.is_null());
        Vec::from_raw_parts(pointer, 1, 1);
    }
}
