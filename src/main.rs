#![no_std]
#![no_main]

mod vga_buffer;
use ironarm::vga_buffer::{update_mouse, enable_mouse};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    ironarm::init();
    enable_mouse();
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("[ok]");
    loop {
        update_mouse();
    }
}
