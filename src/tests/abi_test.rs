use crate::tests::helpers::clear;
use crate::tests::std::alkanes_std_auth_token_build;
use crate::tests::std::alkanes_std_genesis_alkane_build;
use crate::tests::std::alkanes_std_genesis_protorune_build;
use crate::tests::std::alkanes_std_merkle_distributor_build;
use crate::tests::std::alkanes_std_orbital_build;
use crate::tests::std::alkanes_std_owned_token_build;
use crate::tests::std::alkanes_std_proxy_build;
use crate::tests::std::alkanes_std_test_build;
use crate::tests::std::alkanes_std_upgradeable_build;
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
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use wasm_bindgen_test::wasm_bindgen_test;

// Helper function to test the ABI of a contract
fn test_contract_abi(
    contract_name: &str,
    contract_bytes: Vec<u8>,
    expected_methods: Vec<(&str, u128, Vec<&str>)>,
) -> Result<()> {
    let context = Arc::new(Mutex::new(AlkanesRuntimeContext::default()));

    // Create a new instance of the contract
    let mut instance = AlkanesInstance::from_alkane(context, Arc::new(contract_bytes), 100000000)?;

    // Call the __meta function to get the ABI
    let abi_bytes = instance.call_meta()?;

    // Convert the ABI bytes to a string and parse as JSON
    let abi_string = String::from_utf8(abi_bytes.clone())?;
    let abi_json: Value = serde_json::from_slice(&abi_bytes)?;

    // Print the ABI for debugging
    println!("{} ABI: {}", contract_name, abi_string);

    // Verify the contract name
    assert_eq!(abi_json["contract"], contract_name);

    // Verify that methods array exists
    assert!(abi_json["methods"].is_array());
    let methods = abi_json["methods"].as_array().unwrap();

    // Verify that we have the expected number of methods
    assert_eq!(
        methods.len(),
        expected_methods.len(),
        "Unexpected number of methods in ABI for {}",
        contract_name
    );

    // Verify each method
    for (expected_name, expected_opcode, expected_params) in expected_methods {
        // Find the method in the ABI
        let method = methods
            .iter()
            .find(|m| m["name"] == expected_name)
            .unwrap_or_else(|| {
                panic!(
                    "Method {} not found in ABI for {}",
                    expected_name, contract_name
                )
            });

        // Verify the opcode
        assert_eq!(
            method["opcode"].as_u64().unwrap() as u128,
            expected_opcode,
            "Incorrect opcode for method {} in {}",
            expected_name,
            contract_name
        );

        // Verify the parameters
        let params = method["params"].as_array().unwrap();
        assert_eq!(
            params.len(),
            expected_params.len(),
            "Incorrect number of parameters for method {} in {}",
            expected_name,
            contract_name
        );

        // Verify each parameter type
        for (i, expected_param) in expected_params.iter().enumerate() {
            assert_eq!(
                params[i].as_str().unwrap(),
                *expected_param,
                "Incorrect parameter type at index {} for method {} in {}",
                i,
                expected_name,
                contract_name
            );
        }
    }

    Ok(())
}

#[wasm_bindgen_test]
fn test_owned_token_abi() -> Result<()> {
    clear();

    // Expected methods with their opcodes and parameter counts
    let expected_methods = vec![
        ("initialize", 0, vec!["u128", "u128"]),
        ("mint", 77, vec!["u128"]),
        ("get_name", 99, vec![]),
        ("get_symbol", 100, vec![]),
        ("get_total_supply", 101, vec![]),
        ("get_data", 1000, vec![]),
    ];

    test_contract_abi(
        "OwnedToken",
        alkanes_std_owned_token_build::get_bytes(),
        expected_methods,
    )
}

#[wasm_bindgen_test]
fn test_auth_token_abi() -> Result<()> {
    clear();

    // Expected methods with their opcodes and parameter counts
    let expected_methods = vec![
        ("initialize", 0, vec!["u128"]),
        ("authenticate", 1, vec![]),
        ("get_name", 99, vec![]),
        ("get_symbol", 100, vec![]),
    ];

    test_contract_abi(
        "AuthToken",
        alkanes_std_auth_token_build::get_bytes(),
        expected_methods,
    )
}

#[wasm_bindgen_test]
fn test_proxy_abi() -> Result<()> {
    clear();

    // Expected methods with their opcodes and parameter counts
    let expected_methods = vec![
        ("initialize", 0, vec![]),
        ("call_witness", 1, vec!["u128"]),
        ("delegatecall_witness", 2, vec!["u128"]),
        ("call_inputs", 3, vec![]),
        ("delegatecall_inputs", 4, vec![]),
    ];

    test_contract_abi(
        "Proxy",
        alkanes_std_proxy_build::get_bytes(),
        expected_methods,
    )
}

#[wasm_bindgen_test]
fn test_upgradeable_abi() -> Result<()> {
    clear();

    // Expected methods with their opcodes and parameter counts
    let expected_methods = vec![
        ("initialize", 0x7fff, vec!["u128", "u128", "u128"]),
        ("upgrade", 0x7ffe, vec!["u128", "u128"]),
        ("delegate", 0x7ffd, vec![]),
    ];

    test_contract_abi(
        "Upgradeable",
        alkanes_std_upgradeable_build::get_bytes(),
        expected_methods,
    )
}

#[wasm_bindgen_test]
fn test_logger_alkane_abi() -> Result<()> {
    clear();

    // Expected methods with their opcodes and parameter counts
    let expected_methods = vec![
        ("self_call", 2, vec![]),
        ("check_incoming", 3, vec![]),
        ("mint_tokens", 4, vec![]),
        ("return_data_1", 5, vec![]),
        ("get_transaction", 50, vec![]),
        ("hash_loop", 78, vec![]),
        ("return_default_data", 99, vec![]),
    ];

    test_contract_abi(
        "LoggerAlkane",
        alkanes_std_test_build::get_bytes(),
        expected_methods,
    )
}

#[wasm_bindgen_test]
fn test_orbital_abi() -> Result<()> {
    clear();

    // Expected methods with their opcodes and parameter counts
    let expected_methods = vec![
        ("initialize", 0, vec![]),
        ("get_name", 99, vec![]),
        ("get_symbol", 100, vec![]),
        ("get_total_supply", 101, vec![]),
        ("get_data", 1000, vec![]),
    ];

    test_contract_abi(
        "Orbital",
        alkanes_std_orbital_build::get_bytes(),
        expected_methods,
    )
}

#[wasm_bindgen_test]
fn test_merkle_distributor_abi() -> Result<()> {
    clear();

    // Expected methods with their opcodes and parameter counts
    let expected_methods = vec![
        ("initialize", 0, vec!["u128", "u128"]),
        ("claim", 1, vec![]),
    ];

    test_contract_abi(
        "MerkleDistributor",
        alkanes_std_merkle_distributor_build::get_bytes(),
        expected_methods,
    )
}

#[wasm_bindgen_test]
fn test_genesis_alkane_abi() -> Result<()> {
    clear();

    // Expected methods with their opcodes and parameter counts
    let expected_methods = vec![
        ("initialize", 0, vec![]),
        ("mint", 77, vec![]),
        ("get_name", 99, vec![]),
        ("get_symbol", 100, vec![]),
        ("get_total_supply", 101, vec![]),
    ];

    test_contract_abi(
        "GenesisAlkane",
        alkanes_std_genesis_alkane_build::get_bytes(),
        expected_methods,
    )
}

#[wasm_bindgen_test]
fn test_genesis_protorune_abi() -> Result<()> {
    clear();

    // Expected methods with their opcodes and parameter counts
    let expected_methods = vec![
        ("initialize", 0, vec![]),
        ("mint", 77, vec![]),
        ("get_name", 99, vec![]),
        ("get_symbol", 100, vec![]),
        ("get_total_supply", 101, vec![]),
    ];

    test_contract_abi(
        "GenesisProtorune",
        alkanes_std_genesis_protorune_build::get_bytes(),
        expected_methods,
    )
}
