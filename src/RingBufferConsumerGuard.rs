// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


/// Use this to consume.
///
/// When dropped, the data in the `buffer_slice` is "released" (dequeued completely) in a burst.
#[derive(Debug)]
pub struct RingBufferConsumerGuard<'a, T: 'a + Copy>
{
	/// Buffer slice to consume.
	pub buffer_slice: &'a [T],
	
	consumer: &'a RingBufferConsumer<T>,
}

impl<'a, T: 'a + Copy> Drop for RingBufferConsumerGuard<'a, T>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		self.consumer.release(self.buffer_slice.len() as u64)
	}
}

impl<'a, T: 'a + Copy> RingBufferConsumerGuard<'a, T>
{
	/// Use this to release less than all the messages in the `buffer_slice`.
	#[inline(always)]
	pub fn release_fewer(self, fewer: usize)
	{
		debug_assert!(fewer <= self.buffer_slice.len());
		
		self.consumer.release(fewer as u64)
	}
}
