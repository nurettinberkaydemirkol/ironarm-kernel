#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

const TOTAL_FRAMES: usize = 1024;

static mut FRAME_USED: [bool; TOTAL_FRAMES] = [false; TOTAL_FRAMES];

pub struct FrameAllocator;

impl FrameAllocator {
    pub fn new() -> Self {
        FrameAllocator
    }

    pub fn allocate_frame(&self) -> Option<usize> {
        unsafe {
            for i in 0..TOTAL_FRAMES {
                if !FRAME_USED[i] {
                    FRAME_USED[i] = true;
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn deallocate_frame(&self, frame: usize) {
        unsafe {
            FRAME_USED[frame] = false;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    let hello = b"Hello World!";
    for (i, &byte) in hello.iter().enumerate() {
        unsafe {
            *vga_buffer.offset((i * 2) as isize) = byte;
            *vga_buffer.offset((i * 2 + 1) as isize) = 0x0f;
        }
    }

    let allocator = FrameAllocator::new();
    if let Some(frame_index) = allocator.allocate_frame() {
        let frame_char = if frame_index < 10 {
            b'0' + frame_index as u8
        } else {
            b'?'
        };

        let pos = hello.len();
        unsafe {
            *vga_buffer.offset((pos * 2) as isize) = frame_char;
            *vga_buffer.offset((pos * 2 + 1) as isize) = 0x0f;
        }
    }

    loop {}
}
