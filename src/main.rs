#![no_std]
#![no_main]

mod vga_buffer;
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
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("[ok]");
    loop {}
}
