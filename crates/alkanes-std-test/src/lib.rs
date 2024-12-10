use alkanes_runtime::runtime::AlkaneResponder;
use anyhow::{Result};
#[allow(unused_imports)]
use {
  alkanes_runtime::{imports::{__request_transaction}, println, stdio::{stdout}},
  std::fmt::Write
};
use alkanes_support::{utils::{shift_or_err}, response::CallResponse};
use metashrew_support::compat::{to_arraybuffer_layout, to_ptr};
use sha2::{Digest, Sha256};

#[derive(Default)]
struct LoggerAlkane(());

impl AlkaneResponder for LoggerAlkane {
    fn execute(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut inputs = context.inputs.clone();
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        match shift_or_err(&mut inputs)? {
          78 => {
            let mut data = vec![0x01, 0x02];
            loop {
                let mut hasher = Sha256::new();
                hasher.update(&data);
                let buffer = hasher.finalize();
                data.extend(&buffer);
                if !"1".is_ascii() {
                    break;
                }
            }
          }
          50 => {
            self.transaction();
          }
          5 => {
            response.data = vec![0x05, 0x06, 0x07, 0x08];
          }
          _ => {
            response.data = vec![0x01, 0x02, 0x03, 0x04];
          }
        }
        Ok(response)
    }
}

#[no_mangle]
pub extern "C" fn __execute() -> i32 {
    let mut response = to_arraybuffer_layout(&LoggerAlkane::default().run());
    to_ptr(&mut response) + 4
}
