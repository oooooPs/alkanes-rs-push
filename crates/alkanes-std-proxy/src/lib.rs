use alkanes_runtime::declare_alkane;
use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::utils::shift_or_err;
use alkanes_support::{
    cellpack::Cellpack, context::Context, parcel::AlkaneTransfer, response::CallResponse,
    witness::find_witness_payload,
};
use anyhow::{anyhow, Result};
use bitcoin::blockdata::transaction::Transaction;
use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};
use protorune_support::utils::consensus_decode;

#[derive(Default)]
struct Proxy(());

impl Proxy {
    pub fn pull_incoming(&self, context: &mut Context) -> Option<AlkaneTransfer> {
        let i = context
            .incoming_alkanes
            .0
            .iter()
            .position(|v| v.id == context.myself)?;
        Some(context.incoming_alkanes.0.remove(i))
    }
    pub fn only_owner(&self, v: Option<AlkaneTransfer>) -> Result<()> {
        if let Some(auth) = v {
            if auth.value < 1 {
                Err(anyhow!(
                    "must spend a balance of this alkane to the alkane to use as a proxy"
                ))
            } else {
                Ok(())
            }
        } else {
            Err(anyhow!(
                "must spend a balance of this alkane to the alkane to use as a proxy"
            ))
        }
    }
}

fn unwrap_auth(v: Option<AlkaneTransfer>) -> Result<AlkaneTransfer> {
    v.ok_or("")
        .map_err(|_| anyhow!("authentication token not present"))
}

impl AlkaneResponder for Proxy {
    fn execute(&self) -> Result<CallResponse> {
        let mut context = self.context()?;
        let mut inputs = context.inputs.clone();
        let auth = self.pull_incoming(&mut context);
        match shift_or_err(&mut inputs)? {
            0 => {
                if self.load("/initialized".as_bytes().to_vec()).len() != 0 {
                    let mut response: CallResponse = CallResponse::default();
                    response.alkanes = context.incoming_alkanes.clone();
                    response.alkanes.0.push(AlkaneTransfer {
                        id: context.myself.clone(),
                        value: 1,
                    });
                    self.store("/initialized".as_bytes().to_vec(), vec![0x01]);
                    return Ok(response);
                } else {
                    return Err(anyhow!("already initialized"));
                }
            }
            1 => {
                self.only_owner(auth.clone())?;
                let witness_index = shift_or_err(&mut inputs)?;
                let tx =
                    consensus_decode::<Transaction>(&mut std::io::Cursor::new(self.transaction()))?;
                let cellpack = Cellpack::parse(&mut std::io::Cursor::new(
                    find_witness_payload(&tx, witness_index.try_into()?)
                        .ok_or("")
                        .map_err(|_| anyhow!("witness envelope not found"))?,
                ))?;
                let mut response: CallResponse =
                    self.call(&cellpack, &context.incoming_alkanes, self.fuel())?;
                response.alkanes.0.push(unwrap_auth(auth)?);
                Ok(response)
            }
            2 => {
                self.only_owner(auth.clone())?;
                let witness_index = shift_or_err(&mut inputs)?;
                let tx =
                    consensus_decode::<Transaction>(&mut std::io::Cursor::new(self.transaction()))?;
                let cellpack = Cellpack::parse(&mut std::io::Cursor::new(
                    find_witness_payload(&tx, witness_index.try_into()?)
                        .ok_or("")
                        .map_err(|_| anyhow!("witness envelope not found"))?,
                ))?;
                let mut response: CallResponse =
                    self.delegatecall(&cellpack, &context.incoming_alkanes, self.fuel())?;

                response.alkanes.0.push(unwrap_auth(auth)?);
                Ok(response)
            }
            3 => {
                self.only_owner(auth.clone())?;
                let cellpack: Cellpack = inputs.try_into()?;
                let mut response: CallResponse =
                    self.call(&cellpack, &context.incoming_alkanes, self.fuel())?;
                response.alkanes.0.push(unwrap_auth(auth)?);
                Ok(response)
            }
            4 => {
                self.only_owner(auth.clone())?;
                let cellpack: Cellpack = inputs.try_into()?;
                let mut response: CallResponse =
                    self.delegatecall(&cellpack, &context.incoming_alkanes, self.fuel())?;
                response.alkanes.0.push(unwrap_auth(auth)?);
                Ok(response)
            }
            _ => Err(anyhow!("unrecognized opcode")),
        }
    }
}

declare_alkane! {Proxy}
