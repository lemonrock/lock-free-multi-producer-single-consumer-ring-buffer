// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


/// Produces bursts of messages to put into the ring buffer.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RingBufferProducer<T: Copy>
{
	ring_buffer: RingBuffer<T>,
	inner_producer: NonNull<RingBufferProducerInner>,
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
	pub fn acquire<'a>(&'a self, count: u64) -> Option<RingBufferProducerGuard<'a, T>>
	{
		let length_in_bytes = count * RingBuffer::<T>::t_size();
		
		let offset_in_bytes = self.reference().acquire(self.producer(), length_in_bytes);
		
		match offset_in_bytes
		{
			None => None,
			Some(offset_in_bytes) =>
			{
				Some
				(
					RingBufferProducerGuard
					{
						buffer_slice: self.reference().buffer_consumer_slice_mutable(length_in_bytes, offset_in_bytes),
						producer: self,
					}
				)
			}
		}
	}
	
	#[inline(always)]
	pub(crate) fn produce(&self)
	{
		RingBufferInner::<T>::produce(self.producer())
	}
	
	#[inline(always)]
	fn reference(&self) -> &RingBufferInner<T>
	{
		self.ring_buffer.reference()
	}
	
	#[inline(always)]
	fn producer(&self) -> &mut RingBufferProducerInner
	{
		unsafe { &mut * self.inner_producer.as_ptr() }
	}
}
