// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of lock-free-multi-producer-single-consumer-ring-buffer, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 - 2019 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


/// A ring buffer consumer for receiving lock-less bursts of messages.
///
/// Not particularly cheap to consume from (as it walks all producers) so try to use as few producers as possible and consume as much as possible with each call.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RingBufferConsumer<T: Sized>(RingBuffer<T>);

impl<T: Sized> RingBufferConsumer<T>
{
	/// Get a contiguous range which is ready to be consumed.
	///
	/// Only call this on one thread at a time.
	///
	/// Not particularly cheap (as it walks all producers) so try to take as much as possible.
	#[inline(always)]
	pub fn consume<'a>(&'a self) -> RingBufferConsumerGuard<'a, T>
	{
		let (count, offset) = self.reference().consume();
		
		RingBufferConsumerGuard
		{
			buffer_slice: self.reference().buffer_consumer_slice_reference(count, offset),
			release_count: count,
			consumer: self,
		}
	}
	
	#[inline(always)]
	pub(crate) fn release(&self, count: usize)
	{
		self.reference().release(count)
	}
	
	#[inline(always)]
	fn reference(&self) -> &RingBufferInner<T>
	{
		self.0.reference()
	}
}
