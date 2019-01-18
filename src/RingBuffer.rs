// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of lock-free-multi-producer-single-consumer-ring-buffer, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 - 2019 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


/// A ring buffer for sending lock-less bursts of messages.
///
/// Not particularly cheap to consume from (as it walks all producers) so try to use as few producers as possible and consume as much as possible with each call.
///
/// Multi-Producer, Single-Consumer (MP-SC).
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct RingBuffer<T: Copy>
{
	ring_buffer_inner_non_null: NonNull<RingBufferInner<T>>,
	inner_drop_handle: Arc<RingBufferInnerDropHandler<T>>,
	marker: PhantomData<T>,
}

impl<T: Copy> RingBuffer<T>
{
	/// Creates a new ring buffer and returns a consumer to it and producers for it.
	///
	/// When the last consumer or producer is dropped, the ring buffer is freed.
	#[inline(always)]
	pub fn new(capacity: usize, number_of_producers: usize) -> (RingBufferConsumer<T>, Vec<RingBufferProducer<T>>)
	{
		let ring_buffer_inner_non_null = RingBufferInner::allocate(capacity, number_of_producers);

		let ring_buffer = Self
		{
			ring_buffer_inner_non_null,
			inner_drop_handle: Arc::new(RingBufferInnerDropHandler(ring_buffer_inner_non_null)),
			marker: PhantomData,
		};

		let mut ring_buffer_producer_inner_non_null = ring_buffer.reference().first_ring_buffer_producer_inner_non_null();
		let mut producers = Vec::with_capacity(number_of_producers);
		for _ in 0 .. number_of_producers
		{
			producers.push
			(
				RingBufferProducer
				{
					ring_buffer: ring_buffer.clone(),
					ring_buffer_producer_inner_non_null,
				}
			);
			ring_buffer_producer_inner_non_null = RingBufferInner::<T>::next_ring_buffer_producer_inner_non_null(ring_buffer_producer_inner_non_null);
		}

		(RingBufferConsumer(ring_buffer), producers)
	}
	
	#[inline(always)]
	pub(crate) fn reference(&self) -> &RingBufferInner<T>
	{
		unsafe { & * self.ring_buffer_inner_non_null.as_ptr() }
	}
}
