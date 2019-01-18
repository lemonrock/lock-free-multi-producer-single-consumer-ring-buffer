// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of lock-free-multi-producer-single-consumer-ring-buffer, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 - 2019 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


/// Use this to produce.
///
/// When dropped, the data owned by this guard is "produced" (enqueued) in a burst.
///
/// Use `DerefMut` to access the data slice owned by this guard.
///
/// This data is considered to be ***uninitialized***.
#[derive(Debug)]
pub struct RingBufferProducerGuard<'a, T: 'a + Copy>
{
	/// Buffer slice to produce.
	///
	/// All indices ***MUST*** be populated with valid (initialized) data.
	buffer_slice: &'a mut [T],
	
	producer: &'a RingBufferProducer<T>,
}

impl<'a, T: 'a + Copy> Deref for RingBufferProducerGuard<'a, T>
{
	type Target = [T];

	#[inline(always)]
	fn deref(&self) -> &Self::Target
	{
		self.buffer_slice
	}
}

impl<'a, T: 'a + Copy> DerefMut for RingBufferProducerGuard<'a, T>
{
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target
	{
		self.buffer_slice
	}
}

impl<'a, T: 'a + Copy> Drop for RingBufferProducerGuard<'a, T>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		self.producer.produce()
	}
}
