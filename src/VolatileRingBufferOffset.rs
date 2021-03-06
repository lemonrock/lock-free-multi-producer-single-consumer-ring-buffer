// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of lock-free-multi-producer-single-consumer-ring-buffer, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 - 2019 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


#[derive(Debug)]
struct VolatileRingBufferOffset(UnsafeCell<RingBufferOffset>);

impl VolatileRingBufferOffset
{
	/// `x = self`.
	#[inline(always)]
	pub(crate) fn read(&self) -> RingBufferOffset
	{
		unsafe { (self.unsafe_cell_contents() as *const RingBufferOffset).read_volatile() }
	}
	
	/// `self = value`.
	#[inline(always)]
	pub(crate) fn write(&mut self, value: RingBufferOffset)
	{
		unsafe { self.unsafe_cell_contents().write_volatile(value) }
	}
	
	/// `self &= and_equals_value`.
	#[inline(always)]
	pub(crate) fn and_equals(&mut self, and_equals_value: RingBufferOffset)
	{
		let value = self.read() & and_equals_value;
		self.write(value)
	}
	
	#[inline(always)]
	pub(crate) fn atomic_compare_and_exchange_weak(&mut self, old: RingBufferOffset, source: RingBufferOffset) -> bool
	{
		let (_new, ok) = unsafe { atomic_cxchgweak(self.unsafe_cell_contents(), old, source) };
		ok
	}

	#[inline(always)]
	fn unsafe_cell_contents(&self) -> *mut RingBufferOffset
	{
		self.0.get()
	}
}
