// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of lock-free-multi-producer-single-consumer-ring-buffer, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 - 2019 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


/// Use this to consume.
///
/// When dropped, the data held is "released" (dequeued completely) in a burst.
/// Any unread data (eg because one moved out less than the full amount, or didn't finish iterating) is returned to the queue and can be read again.
#[derive(Debug)]
pub struct RingBufferConsumerGuard<'a, T: 'a + Sized>
{
	buffer_slice: &'a [T],

	release_count: usize,

	consumer: &'a RingBufferConsumer<T>,
}

impl<'a, T: 'a + Sized> Drop for RingBufferConsumerGuard<'a, T>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		self.consumer.release(self.release_count)
	}
}

impl<'a, T: 'a + Sized> Iterator for RingBufferConsumerGuard<'a, T>
{
	type Item = T;

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item>
	{
		if self.is_empty()
		{
			return None
		}

		let next = unsafe { transmute_copy(self.buffer_slice.get_unchecked(self.release_count)) };

		self.release_count += 1;

		Some(next)
	}
}

impl<'a, T: 'a + Sized> RingBufferConsumerGuard<'a, T>
{
	/// Is the inner slice empty?
	///
	/// This property is not constant; it can change after calls to `Iterator::next()`, `self.move_into_slice()` and `self.move_into_slice_unsafe()`.
	#[inline(always)]
	pub fn is_empty(&self) -> bool
	{
		self.buffer_slice.len() == self.release_count
	}

	/// Data in this slice.
	///
	/// This property is not constant; it can change after calls to `Iterator::next()`, `self.move_into_slice()` and `self.move_into_slice_unsafe()`.
	#[inline(always)]
	pub fn len(&self) -> usize
	{
		self.buffer_slice.len() - self.release_count
	}

	#[inline(always)]
	fn current_buffer_slice(&self) -> &[T]
	{
		&self.buffer_slice[self.release_count ..]
	}

	/// This moves the data in the buffer to a destination slice using an iteration.
	///
	/// If the destination slice is smaller than this one, then no harm can happen; the excess elements are dropped.
	#[inline(always)]
	pub fn move_into_slice(&mut self, slice: &mut [T])
	{
		let count = slice.len();
		for index in 0 .. count
		{
			unsafe
			{
				let slot = slice.get_unchecked_mut(index);
				*slot = transmute_copy(self.current_buffer_slice().get_unchecked(index));
			}
		}

		self.release_count += count;
	}

	/// This moves the data in the buffer to a destination slice using a `memcpy`; as a consequence, ***any existing data in the slice is NOT dropped***.
	///
	/// If the destination slice is smaller than this one, then no harm can happen; the underlying slice's length is adjusted.
	#[inline(always)]
	pub unsafe fn move_into_slice_unsafe(&mut self, slice: &mut [T])
	{
		let count = slice.len();
		let first = self.current_buffer_slice().as_ptr();
		first.copy_to_nonoverlapping(slice.as_mut_ptr(), count);

		self.release_count += count;
	}

	/// This moves the data in the buffer to a `Box<[T]>` using a `memcpy`.
	///
	/// If the destination slice is smaller than this one, then no harm can happen; the excess elements are dropped.
	///
	/// If it is desired to move out all elements, then `maximum_to_move_out` should be `self.len()`.
	#[inline(always)]
	pub fn move_out(&mut self, maximum_to_move_out: usize) -> Box<[T]>
	{
		let count = min(maximum_to_move_out, self.len());

		let mut boxed_slice =
		{
			let mut vec = Vec::with_capacity(count);
			unsafe { vec.set_len(count) };
			vec.into_boxed_slice()
		};

		let first = self.current_buffer_slice().as_ptr();
		unsafe { first.copy_to_nonoverlapping(boxed_slice.as_mut_ptr(), count) };

		self.release_count += count;

		boxed_slice
	}
}
