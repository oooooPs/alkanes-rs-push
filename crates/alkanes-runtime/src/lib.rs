pub mod auth;
#[cfg(feature = "panic-hook")]
pub mod compat;
pub mod imports;
pub mod message;
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

    (impl AlkaneResponder for $struct_name:ident {
        type Message = $message_type:ident;
    }) => {
        #[no_mangle]
        pub extern "C" fn __execute() -> i32 {
            use alkanes_runtime::runtime::AlkaneResponder;
            use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};

            let mut context = $struct_name::default().context().unwrap();
            let mut inputs = context.inputs.clone();

            if inputs.is_empty() {
                return handle_error("No opcode provided");
            }

            let opcode = inputs[0];
            inputs.remove(0);

            let result = match $message_type::from_opcode(opcode, inputs) {
                Ok(message) => message.dispatch(&$struct_name::default()),
                Err(err) => Err(anyhow::anyhow!("Failed to parse message: {}", err)),
            };

            let response = match result {
                Ok(res) => res,
                Err(err) => {
                    return handle_error(&format!("Error: {}", err));
                }
            };

            // Convert CallResponse to Vec<u8> using serialize
            let serialized_response = response.serialize();
            let response_bytes = to_arraybuffer_layout(&serialized_response);
            Box::leak(Box::new(response_bytes)).as_mut_ptr() as usize as i32 + 4
        }

        #[no_mangle]
        pub extern "C" fn __meta() -> i32 {
            let abi = $message_type::export_abi();
            export_bytes(&abi)
        }

        fn handle_error(msg: &str) -> i32 {
            use alkanes_support::response::CallResponse;
            use metashrew_support::compat::to_arraybuffer_layout;

            // Create a default response with error message in data field
            let mut error_response = CallResponse::default();
            error_response.data = msg.as_bytes().to_vec();

            // Convert CallResponse to Vec<u8> using serialize
            let serialized_response = error_response.serialize();
            let response_bytes = to_arraybuffer_layout(&serialized_response);
            Box::leak(Box::new(response_bytes)).as_mut_ptr() as usize as i32 + 4
        }

        fn export_bytes(data: &[u8]) -> i32 {
            let response_bytes = to_arraybuffer_layout(data);
            Box::leak(Box::new(response_bytes)).as_mut_ptr() as usize as i32 + 4
        }
    };
}
