#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(default_alloc_error_handler)]

#[macro_use]
mod devices;
mod config;
mod mm;
mod panic;
mod sbi;

extern crate alloc;
extern crate bitflags;

use alloc::string::*;
use alloc::*;
use buddy_system_allocator::LockedHeap;
use core::arch::global_asm;

#[global_allocator]
static HEAP: LockedHeap<32> = LockedHeap::empty();

global_asm!(
    "
   .section .text.entry
   .globl _start
_start:
    la      sp, boot_stack_top
    j main

   .section .bss.stack
   .globl boot_stack
boot_stack:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
    .section .bss.heap
    .globl heap_start
heap_start:
    .space 4096 * 16
    .globl heap_end
heap_end:
"
);

#[no_mangle]
extern "C" fn main(hartid: usize, dtb_pa: usize) {
    clear_bss();
    init_heap();
    log!("[{}] Hello, world!, {:p}", hartid, dtb_pa as *const u8);

    for i in 0..=1 {
        let result = sbi::hart_get_status(i);
        log!("[{}] Hart ID {} status: {}", hartid, i, result.value);
    }
    unsafe {
        devices::device_tree::init_tree(dtb_pa);
    }
    mm::init();
    sbi::shutdown();
}
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    log!("Clearing BSS!");
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
    log!("sbss: {:p}", sbss as *const u8);
    log!("ebss: {:p}", ebss as *const u8);
}

fn init_heap() {
    extern "C" {
        fn heap_start();
        fn heap_end();
    }
    unsafe {
        log!("heap_start: {:p}", heap_start as *const u8);
        log!("heap_end: {:p}", heap_end as *const u8);
        log!("heap_size: 0x{:x}", heap_end as usize - heap_start as usize);
        HEAP.lock()
            .init(heap_start as usize, heap_end as usize - heap_start as usize);
    }
}
