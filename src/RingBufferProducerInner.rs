// This file is part of lock-free-multi-producer-single-consumer-ring-buffer. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT. No part of lock-free-multi-producer-single-consumer-ring-buffer, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 - 2019 The developers of lock-free-multi-producer-single-consumer-ring-buffer. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/lock-free-multi-producer-single-consumer-ring-buffer/master/COPYRIGHT.


#[derive(Debug)]
struct RingBufferProducerInner
{
	seen_offset: VolatileRingBufferOffset,
}

impl RingBufferProducerInner
{
	const Default: Self = Self
	{
		seen_offset: VolatileRingBufferOffset(UnsafeCell::new(RingBufferInnerHeader::<()>::MaximumOffset)),
	};

	#[inline(always)]
	fn initialize(this: NonNull<Self>)
	{
		unsafe { write(this.as_ptr(), RingBufferProducerInner::Default) }
	}
}
