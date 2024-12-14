use alkanes_runtime::{runtime::AlkaneResponder, storage::StoragePointer, token::Token};
use alkanes_support::utils::shift_or_err;
use alkanes_support::{parcel::AlkaneTransfer, response::CallResponse};
use metashrew_support::compat::{to_arraybuffer_layout, to_ptr};
use metashrew_support::index_pointer::KeyValuePointer;
use anyhow::{Result, anyhow};
use std::sync::Arc;

#[derive(Default)]
pub struct AuthToken(());

impl Token for AuthToken {
    fn name(&self) -> String {
        String::from("AUTH")
    }
    fn symbol(&self) -> String {
        String::from("AUTH")
    }
}

impl AlkaneResponder for AuthToken {
    fn execute(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut inputs = context.inputs.clone();
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());
        match shift_or_err(&mut inputs)? {
            0 => {
                let mut pointer = StoragePointer::from_keyword("/initialized");
                if pointer.get().len() == 0 {
                    let amount = shift_or_err(&mut inputs)?;
                    response.alkanes = context.incoming_alkanes.clone();
                    response.alkanes.0.push(AlkaneTransfer {
                        id: context.myself.clone(),
                        value: amount,
                    });
                    pointer.set(Arc::new(vec![0x01]));
                    Ok(response)
                } else {
                    return Err(anyhow!("already initialized"));
                }
            }
            1 => {
                if context.incoming_alkanes.0.len() != 1 {
                    return Err(anyhow!("did not authenticate with only the authentication token"));
                }
                let transfer = context.incoming_alkanes.0[0].clone();
                if transfer.id != context.myself.clone() {
                    return Err(anyhow!("supplied alkane is not authentication token"));
                }
                if transfer.value < 1 {
                    return Err(anyhow!("less than 1 unit of authentication token supplied to authenticate"));
                }
                response.data = vec![0x01];
                response.alkanes.0.push(transfer);
                Ok(response)
            }
            99 => {
                response.data = self.name().into_bytes().to_vec();
                Ok(response)
            }
            100 => {
                response.data = self.symbol().into_bytes().to_vec();
                Ok(response)
            }
            _ => {
                return Err(anyhow!("unrecognized opcode"));
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn __execute() -> i32 {
    let mut response = to_arraybuffer_layout(&AuthToken::default().run());
    to_ptr(&mut response) + 4
}
