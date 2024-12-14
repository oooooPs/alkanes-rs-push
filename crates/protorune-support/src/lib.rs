pub mod balance_sheet;
pub mod byte_utils;
pub mod constants;
pub mod proto;
pub mod protostone;
pub mod rune_transfer;
pub mod utils;
pub mod network;

use anyhow;
use bitcoin::{Txid, OutPoint};
use bitcoin::hashes::Hash;

impl TryInto<OutPoint> for proto::protorune::Outpoint {
  type Error = anyhow::Error;
  fn try_into(self) -> Result<OutPoint, Self::Error> {
    Ok(OutPoint {
      txid: Txid::from_byte_array(<&[u8] as TryInto<[u8; 32]>>::try_into(&self.txid)?),
      vout: self.vout.into()
    })
  }
}
