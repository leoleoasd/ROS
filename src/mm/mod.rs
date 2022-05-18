mod addresses;
mod frame_alloctor;
mod page_table;

use addresses::*;
use frame_alloctor::*;

use crate::devices::device_tree::DT;

pub fn init() {
    log!("Initializing frames and pages...");
    extern "C" {
        fn ekernel();
    }
    let start = PhysicalAddress::from(ekernel as usize).ceil();
    for node in &DT.get().unwrap().root.children {
        if node.name.starts_with("memory") {
            let reg = node.prop_raw("reg").unwrap();
            let start0: usize = usize::from_be_bytes(reg[0..8].try_into().unwrap());
            let size: usize = usize::from_be_bytes(reg[8..16].try_into().unwrap());
            log!("Memory start: {:X}, size: {:X}", start0, size);
            init_allocator(start, size + start0 - (ekernel as usize));
            log!("Memory initialized.");
            return;
        }
    }
    panic!("memory error");
}

pub fn alloc_mm_test() {
    extern "C" {
        fn ekernel();
    }
    let start = PhysicalAddress::from(ekernel as usize).ceil().0;
    // init_allocator(PhysicalPageNumber(start), 0x20);

    {
        let frames = frame_alloc_multiple(16);
        assert!(frames.is_some());
        let frames = frames.unwrap();
        let ppn = frames.ppn;
        println!("{:?}", &ppn);
        println!("{:?}", PhysicalAddress::from(ppn));
        drop(frames);
    }
    println!("multiple pass");

    {
        let frames = frame_alloc_multiple(16);
        assert!(frames.is_some());
        let frames = frames.unwrap();
        let ppn = frames.ppn;
        println!("{:?}", &ppn);
        drop(frames);
    }
    println!("multiple pass");

    //{
    //	let frame = frame_alloc();
    //	assert!(frame.is_some());
    //	let uframe = frame.unwrap();
    //	assert_eq!(uframe.ppn, PhysicalPageNumber(start + 0));
    //	let frames = frame_alloc_multiple(16);
    //	assert!(frames.is_some());
    //	let uframes = frames.unwrap();
    //	assert_eq!(uframes.ppn, PhysicalPageNumber(start + 1));
    //	let frames2 = frame_alloc_multiple(16);
    //	assert!(frames2.is_none());
    //}
    //println!("overflow pass");

    {
        let frame = frame_alloc();
        assert!(frame.is_some());
        let frames = frame_alloc_multiple(1);
        assert!(frames.is_some());
        let frames = frames.unwrap();
        assert_eq!(frames.ppn, PhysicalPageNumber(start + 1));
        let frame2: Frame = frames.into();
        assert_eq!(frame2.ppn, PhysicalPageNumber(start + 1));
        let frame3 = frame_alloc();
        assert!(frame3.is_some());
        let frame3 = frame3.unwrap();
        assert_eq!(frame3.ppn, PhysicalPageNumber(start + 2));
        let frames2: FrameRegion = frame3.into();
        assert_eq!(frames2.ppn, PhysicalPageNumber(start + 2));
        assert_eq!(frames2.size, 1);
    }
    println!("convert pass");
}
