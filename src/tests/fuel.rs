use crate::{message::AlkaneMessageContext, tests::std::alkanes_std_test_build};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use anyhow::{anyhow, Result};
use bitcoin::OutPoint;
use metashrew_support::utils::consensus_encode;

use crate::index_block;
use crate::tests::helpers::{self as alkane_helpers, assert_binary_deployed_to_id};
use alkane_helpers::clear;
use alkanes::view;
use bitcoin::Witness;
#[allow(unused_imports)]
use metashrew::{
    println,
    stdio::{stdout, Write},
};
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn test_infinite_loop() -> Result<()> {
    clear();
    let block_height = 840_000;
    // Get the LoggerAlkane ID
    let logger_alkane_id = AlkaneId { block: 2, tx: 1 };

    // Create a cellpack to call the process_numbers method (opcode 11)
    let infinite_exec_cellpack = Cellpack {
        target: AlkaneId { block: 1, tx: 0 },
        inputs: vec![20],
    };

    // Initialize the contract and execute the cellpacks
    let mut test_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [alkanes_std_test_build::get_bytes()].into(),
        [infinite_exec_cellpack].into(),
    );

    index_block(&test_block, block_height)?;

    // Verify the binary was deployed correctly
    let _ = assert_binary_deployed_to_id(
        logger_alkane_id.clone(),
        alkanes_std_test_build::get_bytes(),
    );

    let outpoint = OutPoint {
        txid: test_block.txdata.last().unwrap().compute_txid(),
        vout: 3,
    };

    let trace_data = view::trace(&outpoint)?;
    println!("trace: {:?}", trace_data);

    Ok(())
}
