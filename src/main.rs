#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
mod devices;

mod sbi;
mod panic;

use core::arch::global_asm;

// Allocate a stack.
global_asm!("
   .section .text.entry
   .globl _start
_start:
   la sp, boot_stack_top
   call clear_bss
   call main
   .section .bss.stack
   .globl boot_stack
boot_stack:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
");

#[no_mangle]
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
    println!("Clearing BSS!");
    println!("sbss: {:p}", sbss as *const u8);
    println!("ebss: {:p}", ebss as *const u8);
}

#[no_mangle]
fn main() -> ! {
    println!("Hello, world!");
    panic!("panic!");
    // sbi::sbi::shutdown();
}
