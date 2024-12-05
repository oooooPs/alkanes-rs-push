use crate::message::AlkaneMessageContext;
use crate::network::{genesis, is_genesis};
use crate::vm::fuel::set_message_count;
use anyhow::Result;
use bitcoin::blockdata::block::Block;
use ordinals::{Artifact, Runestone};
use protorune::{message::MessageContext, Protorune};
use protorune_support::network::{set_network, NetworkParams};
use protorune_support::protostone::Protostone;

#[cfg(all(
    not(feature = "mainnet"),
    not(feature = "testnet"),
    not(feature = "luckycoin"),
    not(feature = "dogecoin"),
    not(feature = "bellscoin")
))]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("bcrt"),
        p2pkh_prefix: 0x64,
        p2sh_prefix: 0xc4,
    });
}
#[cfg(feature = "mainnet")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("bc"),
        p2sh_prefix: 0x05,
        p2pkh_prefix: 0x00,
    });
}
#[cfg(feature = "testnet")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("tb"),
        p2pkh_hash: 0x6f,
        p2sh_hash: 0xc4,
    });
}
#[cfg(feature = "luckycoin")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("lky"),
        p2pkh_hash: 0x2f,
        p2sh_hash: 0x05,
    });
}

#[cfg(feature = "dogecoin")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("dc"),
        p2pkh_hash: 0x1e,
        p2sh_hash: 0x16,
    });
}
#[cfg(feature = "bellscoin")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("bel"),
        p2pkh_hash: 0x6f,
        p2sh_hash: 0xc4,
    });
}

pub fn index_block(block: &Block, height: u32) -> Result<()> {
    configure_network();
    if is_genesis(height.into()) {
        genesis(&block).unwrap();
    }
    count_alkanes_protomessages(&block);
    Protorune::index_block::<AlkaneMessageContext>(block.clone(), height.into())?;
    Ok(())
}

pub fn count_alkanes_protomessages(block: &Block) {
    let mut count: u64 = 0;
    for tx in &block.txdata {
        if let Some(Artifact::Runestone(ref runestone)) = Runestone::decipher(tx) {
            if let Ok(protostones) = Protostone::from_runestone(runestone) {
                for protostone in protostones {
                    if protostone.protocol_tag == AlkaneMessageContext::protocol_tag()
                        && protostone.message.len() != 0
                    {
                        count = count + 1;
                    }
                }
            }
        }
    }
    set_message_count(count);
}
