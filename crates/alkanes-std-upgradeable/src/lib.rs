use alkanes_runtime::auth::AuthenticatedResponder;
use alkanes_runtime::{runtime::AlkaneResponder, storage::StoragePointer};
use alkanes_support::utils::{shift_or_err, shift_id_or_err};
use alkanes_support::{cellpack::Cellpack, id::AlkaneId, response::CallResponse};
use anyhow::{anyhow, Result};
use metashrew_support::compat::{to_arraybuffer_layout, to_ptr};
use metashrew_support::index_pointer::KeyValuePointer;
use std::sync::Arc;

#[derive(Default)]
pub struct Upgradeable(());

impl Upgradeable {
    pub fn alkane_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/implementation")
    }
    pub fn alkane(&self) -> Result<AlkaneId> {
        Ok(self.alkane_pointer().get().as_ref().clone().try_into()?)
    }
    pub fn set_alkane(&self, v: AlkaneId) {
        self.alkane_pointer()
            .set(Arc::new(<AlkaneId as Into<Vec<u8>>>::into(v)));
    }
}

impl AuthenticatedResponder for Upgradeable {}

impl AlkaneResponder for Upgradeable {
    fn execute(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut inputs = context.inputs.clone();
        let opcode = shift_or_err(&mut inputs)?;
        if opcode == 0x7fff {
            let mut pointer = StoragePointer::from_keyword("/proxy-initialized");
            if pointer.get().len() != 0 {
                self.set_alkane(shift_id_or_err(&mut inputs)?);
                let auth_token_units = shift_or_err(&mut inputs)?;
                let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes);

                response
                    .alkanes
                    .0
                    .push(self.deploy_auth_token(auth_token_units)?);
                pointer.set(Arc::new(vec![0x01]));
                Ok(response)
            } else {
              Err(anyhow!("already initialized"))
            }
        } else if opcode == 0x7ffe {
            self.only_owner()?;
            self.set_alkane(shift_id_or_err(&mut inputs)?);
            Ok(CallResponse::forward(&context.incoming_alkanes))
        } else {
            let cellpack = Cellpack {
                target: self.alkane()?,
                inputs: inputs.clone(),
            };
            Ok(self.delegatecall(&cellpack, &context.incoming_alkanes, self.fuel())?)
        }
    }
}

#[no_mangle]
pub extern "C" fn __execute() -> i32 {
    let mut response = to_arraybuffer_layout(&Upgradeable::default().run());
    to_ptr(&mut response) + 4
}
