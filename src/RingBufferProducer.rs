// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of lock-free-multi-producer-single-consumer-ring-buffer, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 - 2019 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


/// Produces bursts of messages to put into the ring buffer.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RingBufferProducer<T: Copy>
{
	ring_buffer: RingBuffer<T>,
	ring_buffer_producer_inner_non_null: NonNull<RingBufferProducerInner>,
}

impl<T: Copy> RingBufferProducer<T>
{
	/// Request a space of a given `count` in the ring buffer.
	///
	/// Returns a slice offset into the (external) ring buffer for which `count` bytes are available.
	///
	/// Should be released with `produce()`.
	///
	/// Slice data should be treated as uninitialized, not necessarily zero'd.
	///
	/// * `count` should not be zero.
	/// * `count` should not exceed the buffer size.
	#[inline(always)]
	pub fn acquire<'a>(&'a self, count: usize) -> Option<RingBufferProducerGuard<'a, T>>
	{
		match self.reference().acquire(self.producer(), count)
		{
			None => None,

			Some(offset) =>
			{
				Some
				(
					RingBufferProducerGuard
					{
						buffer_slice: self.reference().buffer_consumer_slice_mutable(count, offset),
						producer: self,
					}
				)
			}
		}
	}

	/// A wrapper around acquire, that retries progressively smaller `count`s if acquire fails, eventually reducing to a count of `1` before giving up.
	///
	/// If it gives up, there will be values remaining in `populate_with` on return.
	///
	/// All the other original values in `populate_with` are moved into the ring buffer (and so are not dropped).
	///
	/// `populate_with` can have a `.len()` of zero.
	#[inline(always)]
	pub fn repeatedly_acquire_and_try_to_populate(&self, populate_with: &mut Vec<T>)
	{
		let mut try_to_acquire_count = populate_with.len();

		while populate_with.len() != 0
		{
			match self.acquire(try_to_acquire_count)
			{
				Some(mut slice_guard) => unsafe
				{
					let from = populate_with.len() - try_to_acquire_count;
					slice_guard.as_mut_ptr().copy_from_nonoverlapping(populate_with.get_unchecked(from), try_to_acquire_count);
					populate_with.set_len(from);
				},

				None =>
				{
					let can_not_acquire_a_zero_slice_so_give_up_and_drop_remaining_file_descriptors = try_to_acquire_count == 1;
					if can_not_acquire_a_zero_slice_so_give_up_and_drop_remaining_file_descriptors
					{
						return
					}

					try_to_acquire_count /= 2;
				}
			}
		}
	}
	
	#[inline(always)]
	pub(crate) fn produce(&self)
	{
		let producer = self.producer();
		debug_assert_ne!(producer.seen_offset.read(), RingBufferInnerHeader::<T>::MaximumOffset);

		fence_stores();

		producer.seen_offset.write(RingBufferInnerHeader::<T>::MaximumOffset);
	}
	
	#[inline(always)]
	fn reference(&self) -> &RingBufferInner<T>
	{
		self.ring_buffer.reference()
	}
	
	#[inline(always)]
	fn producer(&self) -> &mut RingBufferProducerInner
	{
		unsafe { &mut * self.ring_buffer_producer_inner_non_null.as_ptr() }
	}
}
