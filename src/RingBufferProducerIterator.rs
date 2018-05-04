// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


/// Hands out unique producers.
///
/// Panics (in debug) if called for more than the `number_of_producers` passed in `Self::new()`.
///
/// Not thread safe.
#[derive(Debug)]
pub struct RingBufferProducerIterator<T: Copy>
{
	ring_buffer: RingBuffer<T>,
	next_inner_ring_buffer_producer: NonNull<RingBufferProducerInner>,
	producers_to_iterate: usize,
}

impl<T: Copy> Iterator for RingBufferProducerIterator<T>
{
	type Item = RingBufferProducer<T>;
	
	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item>
	{
		if self.producers_to_iterate == 0
		{
			return None
		}
		self.producers_to_iterate -= 1;
		
		let inner_producer = self.next_inner_ring_buffer_producer;
		self.next_inner_ring_buffer_producer = RingBufferInner::<T>::next_producer_reference(inner_producer);
		Some
		(
			RingBufferProducer
			{
				ring_buffer: self.ring_buffer.clone(),
				inner_producer,
			}
		)
	}
}
