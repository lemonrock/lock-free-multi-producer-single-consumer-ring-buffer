// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


/// A ring buffer for sending lock-less bursts of messages.
///
/// Not particularly cheap to consume from (as it walks all producers) so try to use as few producers as possible and consume as much as possible with each call.
///
/// Multi-Producer, Single-Consumer (MP-SC).
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct RingBuffer<T: Copy>
{
	inner: NonNull<RingBufferInner<T>>,
	inner_drop_handle: Arc<RingBufferInnerDropHandler<T>>,
	phantom: PhantomData<T>,
}

impl<T: Copy> RingBuffer<T>
{
	/// Creates a new ring buffer.
	#[inline(always)]
	pub fn new(capacity: u64, number_of_producers: usize) -> (RingBufferConsumer<T>, RingBufferProducerIterator<T>)
	{
		let length_in_bytes = capacity * Self::t_size();
		
		let inner = RingBufferInner::allocate(length_in_bytes, number_of_producers);
		let inner_drop_handle = Arc::new(RingBufferInnerDropHandler(inner));
		
		let this = Self
		{
			inner,
			inner_drop_handle,
			phantom: PhantomData,
		};
		
		(
			RingBufferConsumer(this.clone()),
			RingBufferProducerIterator
			{
				next_inner_ring_buffer_producer: this.reference().first_producer_reference(),
				ring_buffer: this,
				producers_to_iterate: number_of_producers,
			}
		)
	}
	
	#[inline(always)]
	pub(crate) fn reference(&self) -> &RingBufferInner<T>
	{
		unsafe { & * self.inner.as_ptr() }
	}
	
	#[inline(always)]
	pub(crate) fn t_size() -> u64
	{
		size_of::<T>() as u64
	}
}
