cfg_select! {
    target_arch = "x86_64" => {
        pub mod x86_64;

        pub mod internal {
            pub use super::x86_64::*;
        }
    }
    _ => { compile_error!("Lavender only supports x86_64 architecture.")}
}

pub mod paging;

pub fn init() {
    unsafe {
        internal::init();
    }
}
