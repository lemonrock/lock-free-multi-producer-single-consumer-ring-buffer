// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of lock-free-multi-producer-single-consumer-ring-buffer, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 - 2019 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


#[derive(Debug)]
#[repr(C)]
struct RingBufferInner<T: Copy>
{
	/// Fixed size field.
	header: RingBufferInnerHeader<T>,

	/// Variable sized field.
	ring_buffer_producer_inners: PhantomData<RingBufferProducerInner>,

	/// Variable sized field.
	buffer: PhantomData<T>,
}

impl<T: Copy> Deref for RingBufferInner<T>
{
	type Target = RingBufferInnerHeader<T>;

	#[inline(always)]
	fn deref(&self) -> &Self::Target
	{
		&self.header
	}
}

impl<T: Copy> DerefMut for RingBufferInner<T>
{
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target
	{
		&mut self.header
	}
}

impl<T: Copy> RingBufferInner<T>
{
	#[inline(always)]
	pub(crate) fn allocate(capacity: usize, number_of_producers: usize) -> NonNull<Self>
	{
		let header = RingBufferInnerHeader::new(capacity, number_of_producers);

		let layout = header.layout();
		let mut this = unsafe
		{
			let raw_this_pointer = Global.alloc(layout).expect("Out of memory");
			NonNull::new_unchecked(raw_this_pointer.as_ptr() as *mut Self)
		};

		{
			let this_mut: &mut Self = unsafe { this.as_mut() };
			this_mut.initialize_header(header);
			let after_last_ring_buffer_producer_inner_non_null = this_mut.initialize_ring_buffer_producer_inners();
			this_mut.initialize_buffer(after_last_ring_buffer_producer_inner_non_null);
		}
		
		fence_stores();
		
		this
	}

	#[inline(always)]
	pub(crate) fn initialize_header(&mut self, header: RingBufferInnerHeader<T>)
	{
		unsafe { write(self.deref_mut(), header) };
	}

	#[inline(always)]
	pub(crate) fn initialize_ring_buffer_producer_inners(&mut self) -> NonNull<RingBufferProducerInner>
	{
		let mut ring_buffer_producer_inner_non_null = self.first_ring_buffer_producer_inner_non_null();
		for _ in 0 .. self.number_of_producers
		{
			RingBufferProducerInner::initialize(ring_buffer_producer_inner_non_null);
			ring_buffer_producer_inner_non_null = Self::next_ring_buffer_producer_inner_non_null(ring_buffer_producer_inner_non_null);
		}
		ring_buffer_producer_inner_non_null
	}

	#[inline(always)]
	pub(crate) fn initialize_buffer(&mut self, after_last_ring_buffer_producer_inner_non_null: NonNull<RingBufferProducerInner>)
	{
		self.header.initialize_buffer(after_last_ring_buffer_producer_inner_non_null);
	}
	
	#[inline(always)]
	pub(crate) fn free(&mut self)
	{
		let layout = self.layout();
		unsafe { Global.dealloc(NonNull::new_unchecked(self as *mut _ as *mut _), layout) }
	}

	#[inline(always)]
	pub(crate) fn consume(&self) -> (usize, usize)
	{
		self.header.consume(self)
	}

	#[inline(always)]
	fn first_ring_buffer_producer_inner_non_null(&self) -> NonNull<RingBufferProducerInner>
	{
		unsafe { NonNull::new_unchecked(&self.ring_buffer_producer_inners as *const PhantomData<RingBufferProducerInner> as *const RingBufferProducerInner as *mut RingBufferProducerInner) }
	}

	#[inline(always)]
	fn next_ring_buffer_producer_inner_non_null(previous_ring_buffer_producer_inner_non_null: NonNull<RingBufferProducerInner>) -> NonNull<RingBufferProducerInner>
	{
		unsafe { NonNull::new_unchecked(previous_ring_buffer_producer_inner_non_null.as_ptr().add(1)) }
	}
}
