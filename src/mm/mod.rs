mod addresses;
mod frame_alloctor;
mod page_table;

use addresses::*;
use frame_alloctor::*;

pub fn alloc_mm_test() {
    init_allocator(PhysicalPageNumber(0), PhysicalPageNumber(32));
	
	{
		let frames = frame_alloc_multiple(16);
		assert!(frames.is_some());
		let frames = frames.unwrap();
		let ppn = frames.ppn;
		println!("{:?}", &ppn);
		drop(frames);
	}
	println!("multiple pass");

	{
		let frame = frame_alloc();
		assert!(frame.is_some());
		let frame = frame.unwrap();
		let ppn = frame.ppn;
		println!("{:?}", &ppn);
		drop(frame);
	}
	println!("sigle pass");
	
	{
		let frame = frame_alloc();
		assert!(frame.is_some());
		let frames = frame_alloc_multiple(16);
		assert!(frames.is_some());
		let frames2 = frame_alloc_multiple(4);
		assert!(frames2.is_some());
		println!("u");
		let uframes = frames.unwrap();
		println!("{:?}", uframes.ppn);
		println!("{:?}", frames2.unwrap().ppn);
		println!("{:?}", frame.unwrap().ppn);
		let frames = frame_alloc_multiple(8);
		assert!(frames.is_some());
		println!("{:?}", frames.unwrap().ppn);
	}
	println!("mixed pass");

	{
		let frame = frame_alloc();
		assert!(frame.is_some());
		let uframe = frame.unwrap();
		assert_eq!(uframe.ppn, PhysicalPageNumber(0));
		let frames = frame_alloc_multiple(16);
		assert!(frames.is_some());
		let uframes = frames.unwrap();
		assert_eq!(uframes.ppn, PhysicalPageNumber(1));
		let frames2 = frame_alloc_multiple(16);
		assert!(frames2.is_none());
	}
	println!("overflow pass");

	{
		let frame = frame_alloc();
		assert!(frame.is_some());
		let frames = frame_alloc_multiple(1);
		assert!(frames.is_some());
		let frames = frames.unwrap();
		assert_eq!(frames.ppn, PhysicalPageNumber(1));
		let frame2: Frame = frames.into();
		assert_eq!(frame2.ppn, PhysicalPageNumber(1));
		let frame3 = frame_alloc();
		assert!(frame3.is_some());
		let frame3 = frame3.unwrap();
		assert_eq!(frame3.ppn, PhysicalPageNumber(2));
		let frames2: FrameRegion = frame3.into();
		assert_eq!(frames2.ppn, PhysicalPageNumber(2));
		assert_eq!(frames2.size, 1);
	}
	println!("convert pass");
}