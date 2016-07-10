#![feature(lang_items)]
#![feature(const_fn)]
#![no_std]

extern crate spin;
extern crate rlibc;

#[macro_use]
mod vga;

#[no_mangle]
pub extern fn kmain() {
    vga::clear_screen();

    println!("Hi!");
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! { loop{} }

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}

