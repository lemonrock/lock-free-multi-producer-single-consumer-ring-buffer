// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct SpinLockBackOff(u8);

impl Default for SpinLockBackOff
{
	#[inline(always)]
	fn default() -> Self
	{
		Self::Initial
	}
}

impl SpinLockBackOff
{
	const Initial: Self = SpinLockBackOff(4);
	
	/// Exponential back-off for the spinning paths.
	#[inline(always)]
	fn back_off(&mut self)
	{
		let original_count = self.0;
		let mut i = original_count;
		while i != 0
		{
			spin_loop_hint();
			i -= 1;
		}
		
		const Maximum: u8 = 128;
		if original_count < Maximum
		{
			self.0 += original_count;
		}
	}
}
