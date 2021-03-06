// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of lock-free-multi-producer-single-consumer-ring-buffer, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2018 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


#![allow(non_upper_case_globals)]
#![deny(missing_docs)]
#![feature(allocator_api, core_intrinsics)]


//! # lock-free-multi-producer-single-consumer-ring-buffer
//!
//! ## Usage
//!
//! ```
//! extern crate lock_free_multi_produce_single_consume_ring_buffer;
//!
//! use ::lock_free_multi_produce_single_consume_ring_buffer::*;
//!
//! let (ring_buffer_consumer, ring_buffer_producers) = RingBuffer::new(capacity, number_of_producers);
//!
//! // For each producer thread.
//! let ring_buffer_producer = ring_buffer_producers.get(0);
//!
//! let result = ring_buffer_producer.acquire(length);
//! // result is `None` if length was too much; try a shorter length.
//!
//! let slice_guard = result.unwrap();
//!
//! // Dereferences to a slice.
//! // slice_guard[0] = some_value;
//!
//! // Produce (relinquishes the slice).
//! drop(slice_guard);
//!
//! ring_buffer_producer.produce();
//!
//! // For each consumer thread.
//! let slice_guard = ring_buffer_consumer.consume();
//!
//! // Iterate, move out, etc.
//! println!("should be `some_value`", slice_guard.move_out(slice_guard.len())[0]);
//!
//! // Releases the slice so producers can now use it.
//! drop(slice_guard);
//! ```
//!
//! Once all the producers and the consumer are dropped then the memory underlying the ring buffer is freed and any unconsumed items in it are safely `Drop`ped.
//!
//!
//! ## The following documentation is originally "Copyright (c) 2016-2017 Mindaugas Rasiukevicius <rmind at noxt eu>".
//!
//! Atomic multi-producer single-consumer ring buffer, which supports contiguous range operations and which can be conveniently used for message passing.
//!
//! There are three offsets ("think of clock hands"):-
//!
//! * `NEXT`: marks the beginning of the available space,
//! * `WRITTEN`: the point up to which the data is actually written.
//! * Observed `READY`: point up to which data is ready to be written.
//!
//!
//! ### Producers
//!
//! Observe and save the 'next' offset, then request `N` bytes from the ring buffer by atomically advancing the `next` offset.
//! Once the data is written into the "reserved" buffer space, the thread clears the saved value; these observed values are used to compute the `ready` offset.
//!
//!
//! ### Consumer
//!
//! Writes the data between `written` and `ready` offsets and updates the `written` value.
//! The consumer thread scans for the lowest seen value by the producers.
//!
//!
//! ### Key invariant
//!
//! Producers cannot go beyond the `written` offset
//!
//! Producers are not allowed to catch up with the consumer.
//!
//! Only the consumer is allowed to catch up with the producer, ie set the `written` offset to be equal to the `next` offset.
//!
//!
//! ### Wrap-around
//!
//! If the producer cannot acquire the requested length due to little available space at the end of the buffer, then it will wraparound.
//!
//! The `WrapLockBit` in `next` offset is used to lock the `end` offset.
//!
//! There is an ABA problem if one producer stalls while a pair of producer and consumer would both successfully wrap-around and set the `next` offset to the stale value of the first producer, thus letting it to perform a successful compare-and-swap (CAS) violating the invariant.
//! A counter in the `next` offset (masked by `WrapCounter`) is used to prevent from this problem.
//! It is incremented on wraparounds.
//!
//! The same ABA problem could also cause a stale `ready` offset, which could be observed by the consumer.
//! The algorithm sets `WrapLockBit` in the `seen` value before advancing the `next` and clears this bit after the successful advancing; this ensures that only the stable `ready` observed by the consumer.


#[macro_use] extern crate likely;


use ::std::alloc::Alloc;
use ::std::alloc::Global;
use ::std::alloc::Layout;
use ::std::cell::Cell;
use ::std::cell::UnsafeCell;
use ::std::cmp::min;
use ::std::cmp::max;
use ::std::intrinsics::atomic_cxchgweak;
use ::std::marker::PhantomData;
use ::std::mem::align_of;
use ::std::mem::size_of;
use ::std::mem::transmute_copy;
use ::std::mem::uninitialized;
use ::std::ops::Deref;
use ::std::ops::DerefMut;
use ::std::ptr::drop_in_place;
use ::std::ptr::NonNull;
use ::std::ptr::write;
use ::std::slice::from_raw_parts;
use ::std::slice::from_raw_parts_mut;
use ::std::sync::atomic::fence;
use ::std::sync::atomic::Ordering::SeqCst;
use ::std::sync::atomic::spin_loop_hint;
use std::sync::Arc;


include!("fence_stores.rs");
include!("RingBuffer.rs");
include!("RingBufferConsumer.rs");
include!("RingBufferConsumerGuard.rs");
include!("RingBufferInner.rs");
include!("RingBufferInnerHeader.rs");
include!("RingBufferInnerDropHandler.rs");
include!("RingBufferOffset.rs");
include!("RingBufferProducer.rs");
include!("RingBufferProducerGuard.rs");
include!("RingBufferProducerInner.rs");
include!("SpinLockBackOff.rs");
include!("VolatileRingBufferOffset.rs");
