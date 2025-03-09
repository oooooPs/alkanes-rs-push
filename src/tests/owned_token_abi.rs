use crate::tests::helpers::clear;
use crate::tests::std::alkanes_std_owned_token_build;
use crate::vm::fuel::FuelTank;
use crate::vm::instance::AlkanesInstance;
use crate::vm::runtime::AlkanesRuntimeContext;
use alkanes::vm::fuel::VirtualFuelBytes;
use anyhow::Result;
#[allow(unused_imports)]
use metashrew::{
    println,
    stdio::{stdout, Write},
};
use protorune::test_helpers::create_block_with_coinbase_tx;
use std::sync::{Arc, Mutex};
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn test_owned_token_abi() -> Result<()> {
    clear();
    let context = Arc::new(Mutex::new(AlkanesRuntimeContext::default()));
    // Create a new instance of the OwnedToken contract
    let mut instance = AlkanesInstance::from_alkane(
        context,
        Arc::new(alkanes_std_owned_token_build::get_bytes()),
        100000000,
    )?;

    // Call the __meta function to get the ABI
    let abi_bytes = instance.call_meta()?;

    // Convert the ABI bytes to a string
    let abi_string = String::from_utf8(abi_bytes)?;

    // Print the ABI for debugging
    println!("OwnedToken ABI: {}", abi_string);

    // Verify that the ABI contains the expected methods
    assert!(abi_string.contains("OwnedToken"));
    assert!(abi_string.contains("initialize"));
    assert!(abi_string.contains("mint"));
    assert!(abi_string.contains("get_name"));
    assert!(abi_string.contains("get_symbol"));
    assert!(abi_string.contains("get_total_supply"));
    assert!(abi_string.contains("get_data"));

    // Verify that the ABI contains the expected opcodes
    assert!(abi_string.contains("\"opcode\": 0")); // initialize
    assert!(abi_string.contains("\"opcode\": 77")); // mint
    assert!(abi_string.contains("\"opcode\": 99")); // get_name
    assert!(abi_string.contains("\"opcode\": 100")); // get_symbol
    assert!(abi_string.contains("\"opcode\": 101")); // get_total_supply
    assert!(abi_string.contains("\"opcode\": 1000")); // get_data

    Ok(())
}
