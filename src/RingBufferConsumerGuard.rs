// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of lock-free-multi-producer-single-consumer-ring-buffer, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 - 2019 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


/// Use this to consume.
///
/// When dropped, the data held is "released" (dequeued completely) in a burst.
///
/// Access the data held as a slice by dereferencing (`eg &self[..]`)
#[derive(Debug)]
pub struct RingBufferConsumerGuard<'a, T: 'a + Copy>
{
	buffer_slice: &'a [T],
	
	consumer: &'a RingBufferConsumer<T>,
}

impl<'a, T: 'a + Copy> Drop for RingBufferConsumerGuard<'a, T>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		self.consumer.release(self.buffer_slice.len())
	}
}

impl<'a, T: 'a + Copy> Deref for RingBufferConsumerGuard<'a, T>
{
	type Target = [T];

	#[inline(always)]
	fn deref(&self) -> &Self::Target
	{
		self.buffer_slice
	}
}

impl<'a, T: 'a + Copy> RingBufferConsumerGuard<'a, T>
{
	/// Use this to release less than all the messages in the `buffer_slice`.
	#[inline(always)]
	pub fn release_fewer(mut self, fewer: usize)
	{
		self.buffer_slice = &self.buffer_slice[0 .. fewer];
		drop(self)
	}
}
