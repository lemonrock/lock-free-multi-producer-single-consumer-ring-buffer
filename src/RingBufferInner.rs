// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


#[derive(Debug)]
#[repr(C)]
struct RingBufferInner<T: Copy>
{
	/// Ring buffer space (in bytes).
	space: u64,
	buffer: NonNull<u8>,
	
	/// The `NEXT` hand is atomically updated by the producer.
	/// `Self::WrapLockBit` is set in case of wrap-around; in such a case the producer can update the `end` offset.
	next: UnsafeCell<VolatileRingBufferOffset>,
	end: Cell<RingBufferOffset>,
	
	/// Updated by consumer.
	written: Cell<RingBufferOffset>,
	
	number_of_producers: usize,
	producers: UnsafeCell<PhantomData<T>>,
}

impl<T: Copy> RingBufferInner<T>
{
	const WrapCounter: u64 = 0x7FFFFFFF00000000;
	
	const WrapLockBit: RingBufferOffset = 0x8000000000000000;
	
	const WrapLockMask: RingBufferOffset = !Self::WrapLockBit;
	
	const MaximumOffset: RingBufferOffset = ::std::u64::MAX & Self::WrapLockMask;
	
	const OffsetMask: RingBufferOffset = 0x00000000FFFFFFFF;
	
	#[inline(always)]
	pub(crate) fn allocate(length_in_bytes: u64, number_of_producers: usize) -> NonNull<Self>
	{
		assert!(length_in_bytes < ::std::usize::MAX as u64, "length_in_bytes '{}' exceeds ::std::usize::MAX '{}'", length_in_bytes, ::std::usize::MAX);
		
		let alignment = Self::alignment() as u64;
		let round_up_length_to_alignment = ((length_in_bytes + alignment - 1) / alignment) * alignment;
		
		let space = round_up_length_to_alignment;
		assert!(space < ::std::usize::MAX as u64, "space '{}' exceeds ::std::usize::MAX '{}'", space, ::std::usize::MAX);
		assert!(space < Self::OffsetMask, "space '{}' equals or exceeds Self::OffsetMask '{}'", space, Self::OffsetMask);
		
		unsafe
		{
			let x = Global.alloc(Self::layout(number_of_producers, space as usize)).expect("Out of memory");
			let mut this = NonNull::new_unchecked(x.as_ptr() as *mut Self);
			
			{
				let this_mut = &mut this.as_mut();
				write(&mut this_mut.space, space);
				write(&mut this_mut.next, UnsafeCell::new(VolatileRingBufferOffset(0)));
				write(&mut this_mut.end, Cell::new(Self::MaximumOffset));
				write(&mut this_mut.written, Cell::new(0));
				write(&mut this_mut.number_of_producers, number_of_producers);
				
				let mut producer_reference = this_mut.first_producer_reference();
				for _ in 0 .. number_of_producers
				{
					write(producer_reference.as_ptr(), RingBufferProducerInner
					{
						seen_offset: VolatileRingBufferOffset(Self::MaximumOffset),
					});
					producer_reference = Self::next_producer_reference(producer_reference);
				}
				
				let last_producer_reference = producer_reference;
				let buffer = NonNull::new_unchecked(last_producer_reference.as_ptr() as *mut u8);
				write(&mut this_mut.buffer, buffer);
			}
			
			Self::fence_stores();
			
			this
		}
	}
	
	#[inline(always)]
	pub(crate) fn free(&mut self)
	{
		let layout = Self::layout(self.number_of_producers, self.space as usize);
		
		unsafe { Global.dealloc(NonNull::new_unchecked(self as *mut _ as *mut _), layout) }
	}
	
	#[inline(always)]
	pub(crate) fn acquire(&self, producer: &mut RingBufferProducerInner, length_in_bytes: u64) -> Option<u64>
	{
		debug_assert_ne!(length_in_bytes, 0, "length_in_bytes can not be zero");
		debug_assert!(length_in_bytes <= self.space, "length_in_bytes '{}' exceeds self.space '{}'", length_in_bytes, self.space);
		debug_assert!(producer.seen_offset.read() == Self::MaximumOffset);
		
		let mut target;
		let mut next;
		
		// This is a do-while loop.
		while
		{
			// Get the stable `next` offset.
			// Save the observed `next` value (i.e. the `seen` offset), but mark the value as unstable (set `Self::WrapLockBit`).
			//
			// Note: CAS will issue a `memory_order_release` for us and thus ensures that it reaches global visibility together with new `next`.
			let seen = self.stable_next_offset();
			next = seen & Self::OffsetMask;
			debug_assert!(next < self.space, "next equals or exceeds space");
			producer.seen_offset.write(next | Self::WrapLockBit);
			
			// Compute the target offset.
			// Key invariant: we cannot go beyond the `WRITTEN` offset or catch up with it.
			target = next + length_in_bytes;
			let written = self.written();
			// The producer must wait.
			if Self::unlikely(next < written && target >= written)
			{
				producer.seen_offset.write(Self::MaximumOffset);
				return None
			}
			
			if Self::unlikely(target >= self.space)
			{
				let exceed = target > self.space;
				
				// Wrap-around and start from the beginning.
				//
				// If we would exceed the buffer, then attempt to acquire the `Self::WrapLockBit` and use the space in the beginning.
				// If we used all space exactly to the end, then reset to 0.
				//
				// Check the invariant again.
				target = if exceed
				{
					Self::WrapLockBit | length_in_bytes
				}
				else
				{
					0
				};
				if (target & Self::OffsetMask) >= written
				{
					producer.seen_offset.write(Self::MaximumOffset);
					return None
				}
				
				// Increment the wrap-around counter.
				target |= (seen + 0x100000000) & Self::WrapCounter;
			}
			else
			{
				// Preserve the wrap-around counter.
				target |= seen & Self::WrapCounter;
			}
			
			!self.next_mutable().atomic_compare_and_exchange_weak(seen, target)
		}
		{
		}
		
		// Acquired the range.
		// Clear `Self::WrapLockBit` in the `seen` value thus indicating that it is now stable.
		producer.seen_offset.and_equals(Self::WrapLockMask);
		
		// If we set the `Self::WrapLockBit` in the `next` (because we exceed the remaining space and need to wrap-around), then save the `end` offset and release the lock.
		if Self::unlikely(target & Self::WrapLockBit != 0)
		{
			// Cannot wrap-around again if consumer did not catch-up.
			debug_assert!(self.written() <= next);
			debug_assert_eq!(self.end(), Self::MaximumOffset);
			self.set_end(next);
			next = 0;
			
			// Unlock: ensure the `end` offset reaches global visibility before the lock is released.
			Self::fence_stores();
			self.next_mutable().write(target & Self::WrapLockMask)
		}
		debug_assert!((target & Self::OffsetMask) <= self.space);
		Some(next)
	}
	
	#[inline(always)]
	pub(crate) fn produce(producer: &mut RingBufferProducerInner)
	{
		debug_assert_ne!(producer.seen_offset.read(), Self::MaximumOffset);
		Self::fence_stores();
		producer.seen_offset.write(Self::MaximumOffset);
	}
	
	#[inline(always)]
	pub(crate) fn consume(&self) -> (u64, u64)
	{
		let mut written = self.written();
		let mut next;
		let mut ready;
		
		'retry: loop
		{
			// Get the stable `next` offset.
			// Note: `self.stable_next_offset()` issued a load memory barrier.
			// The area between the `written` offset and the `next` offset will be the *preliminary* target buffer area to be consumed.
			next = self.stable_next_offset() & Self::OffsetMask;
			// If producers did not advance, then nothing to do.
			if written == next
			{
				return (0, 0)
			}
			
			// Observe the `ready` offset of each producer.
			//
			// At this point, some producer might have already triggered the wrap-around and some (or all) seen `ready` values might be in the range between 0 and `written`.
			// We have to skip them.
			ready = Self::MaximumOffset;
			
			let mut producer = self.first_producer_reference();
			'workers: for _ in 0 .. self.number_of_producers
			{
				let mut seen_offset;
				
				// Get a stable `seen` value.
				// This is necessary since we want to discard the stale `seen` values.
				let mut spin_lock_back_off = SpinLockBackOff::Initial;
				while
				{
					seen_offset = unsafe { producer.as_ref() }.seen_offset.read();
					seen_offset & Self::WrapLockBit != 0
				}
				{
					spin_lock_back_off.back_off();
				}
				
				// Ignore the offsets after the possible wrap-around.
				// We are interested in the smallest seen offset that is not behind the `written` offset.
				if seen_offset >= written
				{
					ready = min(seen_offset, ready);
				}
				debug_assert!(ready >= written);
				
				producer = RingBufferInner::<T>::next_producer_reference(producer);
			}
			
			// Finally, we need to determine whether wrap-around occurred and deduct the safe `ready` offset.
			if next < written
			{
				let end = min(self.space, self.end());
				
				// Wrap-around case.
				// Check for the cut off first.
				//
				// Reset the `written` offset if it reached the end of the buffer or the `end` offset (if set by a producer).
				// However, we must check that the producer is actually done (the observed `ready` offsets are clear).
				if ready == Self::MaximumOffset && written == end
				{
					// Clear the 'end' offset if was set.
					if self.end() != Self::MaximumOffset
					{
						self.set_end(Self::MaximumOffset);
						Self::fence_stores();
					}
					
					// Wrap-around the consumer and start from zero.
					written = 0;
					self.set_written(0);
					continue 'retry;
				}
				
				// We cannot wrap-around yet; there is data to consume at the end.
				// The ready range is smallest of the observed `ready` or the `end` offset.
				// If neither is set, then the actual end of the buffer.
				debug_assert!(ready > next);
				ready = min(ready, end);
				debug_assert!(ready >= written);
			}
			else
			{
				// Regular case.
				//
				// Up to the observed `ready` (if set) or the `next` offset.
				ready = min(ready, next);
			}
			
			let to_write = ready - written;
			let offset = written;
			
			debug_assert!(ready >= written);
			debug_assert!(to_write <= self.space);
			
			return (to_write, offset)
		}
	}
	
	#[inline(always)]
	pub(crate) fn release(&self, number_of_bytes: u64)
	{
		debug_assert!(self.written() <= self.space);
		debug_assert!(self.written() <= self.end());
		
		let number_written = self.written() + number_of_bytes;
		debug_assert!(number_written <= self.space);
		
		let value = if number_written == self.space
		{
			0
		}
		else
		{
			number_written
		};
		self.set_written
		(
			value
		);
	}
	
	#[inline(always)]
	pub(crate) fn buffer_consumer_slice_reference(&self, count_in_bytes: u64, offset_in_bytes: u64) -> &[T]
	{
		let pointer = self.buffer_pointer(offset_in_bytes) as *const T;
		unsafe { from_raw_parts(pointer, Self::buffer_length(count_in_bytes)) }
	}
	
	#[inline(always)]
	pub(crate) fn buffer_consumer_slice_mutable(&self, count_in_bytes: u64, offset_in_bytes: u64) -> &mut [T]
	{
		let pointer = self.buffer_pointer(offset_in_bytes) as *mut T;
		unsafe { from_raw_parts_mut(pointer, Self::buffer_length(count_in_bytes)) }
	}
	
	#[inline(always)]
	fn buffer_pointer(&self, offset_in_bytes: u64) -> u64
	{
		(self.buffer.as_ptr() as u64) + offset_in_bytes
	}
	
	#[inline(always)]
	fn buffer_length(count_in_bytes: u64) -> usize
	{
		(count_in_bytes / RingBuffer::<T>::t_size()) as usize
	}
	
	#[inline(always)]
	fn layout(number_of_producers: usize, space: usize) -> Layout
	{
		Layout::from_size_align(Self::header_size() + Self::producers_size(number_of_producers) + space, Self::alignment()).unwrap()
	}
	
	#[inline(always)]
	fn alignment() -> usize
	{
		max(align_of::<RingBufferProducerInner>(), max(align_of::<Self>(), align_of::<T>()))
	}
	
	#[inline(always)]
	fn header_size() -> usize
	{
		size_of::<Self>()
	}
	
	#[inline(always)]
	fn producer_size() -> usize
	{
		size_of::<RingBufferProducerInner>()
	}
	
	#[inline(always)]
	fn producers_size(producer_index: usize) -> usize
	{
		producer_index * Self::producer_size()
	}
	
	#[inline(always)]
	fn first_producer_reference(&self) -> NonNull<RingBufferProducerInner>
	{
		unsafe { NonNull::new_unchecked(self.producers.get() as *mut RingBufferProducerInner) }
	}
	
	#[inline(always)]
	fn next_producer_reference(producer: NonNull<RingBufferProducerInner>) -> NonNull<RingBufferProducerInner>
	{
		unsafe { NonNull::new_unchecked(((producer.as_ptr() as *mut RingBufferProducerInner as usize) + size_of::<*mut RingBufferProducerInner>()) as *mut RingBufferProducerInner) }
	}
	
	#[inline(always)]
	fn stable_next_offset(&self) -> RingBufferOffset
	{
		let mut next;
		
		let mut spin_lock_back_off = SpinLockBackOff::Initial;
		while
		{
			next = self.next().read();
			next & Self::WrapLockBit != 0
		}
		{
			spin_lock_back_off.back_off();
		}
		
		Self::fence_loads();
		
		debug_assert!((next & Self::MaximumOffset) < self.space);
		next
	}
	
	#[inline(always)]
	fn next(&self) -> &VolatileRingBufferOffset
	{
		unsafe { & * self.next.get() }
	}
	
	#[inline(always)]
	fn next_mutable(&self) -> &mut VolatileRingBufferOffset
	{
		unsafe { &mut * self.next.get() }
	}
	
	#[inline(always)]
	fn written(&self) -> RingBufferOffset
	{
		self.written.get()
	}
	
	#[inline(always)]
	fn set_written(&self, written: RingBufferOffset)
	{
		self.written.set(written);
	}
	
	#[inline(always)]
	fn end(&self) -> RingBufferOffset
	{
		self.end.get()
	}
	
	#[inline(always)]
	fn set_end(&self, end: RingBufferOffset)
	{
		self.end.set(end);
	}
	
	#[inline(always)]
	fn fence_stores()
	{
		fence(SeqCst)
	}
	
	#[inline(always)]
	fn fence_loads()
	{
		fence(SeqCst)
	}
	
	#[inline(always)]
	fn unlikely(value: bool) -> bool
	{
		unsafe { unlikely(value) }
	}
}
