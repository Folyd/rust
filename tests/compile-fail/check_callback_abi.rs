#![feature(core_intrinsics)]

extern "C" fn try_fn(_: *mut u8) {
    unreachable!();
}

fn main() {
    unsafe {
        std::intrinsics::r#try( //~ ERROR calling a function with ABI C using caller ABI Rust
            std::mem::transmute::<extern "C" fn(*mut u8), _>(try_fn),
            std::ptr::null_mut(),
            |_, _| unreachable!(),
        );
    }
}
