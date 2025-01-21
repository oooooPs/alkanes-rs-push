pub mod auth;
#[cfg(feature = "panic-hook")]
pub mod compat;
pub mod imports;
pub mod runtime;
pub mod stdio;
pub mod storage;
pub mod token;
pub use crate::stdio::stdout;

use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};

#[macro_export]
macro_rules! declare_alkane {
    ($struct_name:ident) => {
        #[no_mangle]
        pub extern "C" fn __execute() -> i32 {
            let mut response = to_arraybuffer_layout(&$struct_name::default().run());
	    Box::leak(Box::new(response)).as_mut_ptr() as usize as i32 + 4
        }
    };
}
