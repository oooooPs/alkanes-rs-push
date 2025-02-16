use crate::message::AlkaneMessageContext;
use alkanes_support::id::AlkaneId;
use bitcoin::{Transaction, Address, Amount, Block, TxOut, TxIn, OutPoint};
use anyhow::{Result};
use ordinals::Runestone;
use bitcoin::address::NetworkChecked;
use std::str::FromStr;
use metashrew_support::{index_pointer::KeyValuePointer, utils::consensus_encode};
use alkanes::indexer::index_block;
use protorune::protostone::{Protostones};
use protorune::{test_helpers as helpers, balance_sheet::load_sheet, message::MessageContext, tables::RuneTable};
use protorune_support::balance_sheet::{ProtoruneRuneId};
use bitcoin::{ScriptBuf, Sequence, transaction::Version};
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune_support::protostone::{Protostone, ProtostoneEdict};
use alkanes_support::envelope::RawEnvelope;
use crate::tests::std::alkanes_std_test_build;
use crate::tests::helpers as alkane_helpers;
use alkane_helpers::clear;
use metashrew::{println, stdio::{stdout, Write}};
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn test_edict_to_protomessage() -> Result<()> {
    clear();
    let block_height = 840_000;
    let mut test_block: Block = helpers::create_block_with_coinbase_tx(block_height);
    let tx = Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: RawEnvelope::from(alkanes_std_test_build::get_bytes()).to_gzipped_witness()
        }],
        output: vec![TxOut {
            script_pubkey: Address::from_str(ADDRESS1().as_str())
                .unwrap()
                .require_network(get_btc_network())
                .unwrap()
                .script_pubkey(),
            value: Amount::from_sat(100)
        }, TxOut {
            script_pubkey: Address::from_str(ADDRESS1().as_str())
                .unwrap()
                .require_network(get_btc_network())
                .unwrap()
                .script_pubkey(),
            value: Amount::from_sat(100)
        }, TxOut {
            script_pubkey: (Runestone {
                edicts: vec![],
                etching: None,
                mint: None,
                pointer: None,
                protocol: Some(vec![Protostone {
                    message: vec![1, 0, 4],
                    protocol_tag: 1,
                    from: None,
                    burn: None,
                    pointer: Some(6),
                    refund: Some(6),
                    edicts: vec![]
                }, Protostone {
                    message: vec![1, 0, 4],
                    protocol_tag: 1,
                    from: None,
                    burn: None,
                    refund: Some(6),
                    pointer: Some(6),
                    edicts: vec![]
                }, Protostone {
                    message: vec![],
                    protocol_tag: 1,
                    burn: None,
                    from: None,
                    refund: Some(8),
                    pointer: Some(8),
                    edicts: vec![ProtostoneEdict {
                        id: ProtoruneRuneId {
                            block: 2,
                            tx: 1
                        },
                        amount: 100,
                        output: 0
                    }]
                }, Protostone {
                    message: vec![2, 1, 3],
                    protocol_tag: 1,
                    from: None,
                    pointer: Some(1),
                    burn: None,
                    refund: Some(1),
                    edicts: vec![]
                }].encipher()?)
            }).encipher(),
            value: Amount::from_sat(0)
        }]
    };
    test_block.txdata.push(tx);
    index_block(&test_block, block_height)?;
    let edict_outpoint = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 0
    };
    let sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&edict_outpoint)?),
    );
    println!("{:?}", sheet);
    Ok(())
}
