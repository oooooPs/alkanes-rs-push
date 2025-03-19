use crate::tests::helpers::clear;
use crate::vm::instance::AlkanesInstance;
use crate::vm::runtime::AlkanesRuntimeContext;
use alkanes_runtime::{declare_alkane, message::MessageDispatch, runtime::AlkaneResponder};
use alkanes_support::{context::Context, response::CallResponse};
use anyhow::{anyhow, Result};
use metashrew_support::compat::to_arraybuffer_layout;
use std::sync::{Arc, Mutex};
use wasm_bindgen_test::wasm_bindgen_test;

// Define a test contract that uses Vec types
#[derive(Default)]
pub struct VecTest(());

#[derive(MessageDispatch)]
enum VecTestMessage {
    #[opcode(0)]
    Initialize,

    #[opcode(1)]
    ProcessNumbers { numbers: Vec<u128> },

    #[opcode(2)]
    ProcessStrings { strings: Vec<String> },

    #[opcode(3)]
    #[returns(Vec<u128>)]
    GetNumbers,

    #[opcode(4)]
    #[returns(Vec<String>)]
    GetStrings,

    #[opcode(5)]
    ProcessNestedVec { nested: Vec<Vec<u128>> },
}

impl VecTest {
    fn initialize(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        Ok(response)
    }

    fn process_numbers(&self, numbers: Vec<u128>) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Sum the numbers and store in response data
        let sum: u128 = numbers.iter().sum();
        response.data = sum.to_le_bytes().to_vec();

        Ok(response)
    }

    fn process_strings(&self, strings: Vec<String>) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Concatenate the strings and store in response data
        let concat = strings.join(",");
        response.data = concat.into_bytes();

        Ok(response)
    }

    fn get_numbers(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Return a sample vector of numbers
        let numbers = vec![1u128, 2u128, 3u128];
        let mut data = Vec::new();

        // First write the length
        data.extend_from_slice(&(numbers.len() as u128).to_le_bytes());

        // Then write each number
        for num in numbers {
            data.extend_from_slice(&num.to_le_bytes());
        }

        response.data = data;

        Ok(response)
    }

    fn get_strings(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Return a sample vector of strings
        let strings = vec!["hello".to_string(), "world".to_string()];
        let mut data = Vec::new();

        // First write the length
        data.extend_from_slice(&(strings.len() as u128).to_le_bytes());

        // Then write each string with null terminator
        for s in strings {
            let mut bytes = s.into_bytes();
            bytes.push(0); // Null terminator
            data.extend_from_slice(&bytes);
        }

        response.data = data;

        Ok(response)
    }

    fn process_nested_vec(&self, nested: Vec<Vec<u128>>) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Count total elements in the nested vector
        let total_elements: usize = nested.iter().map(|v| v.len()).sum();
        response.data = (total_elements as u128).to_le_bytes().to_vec();

        Ok(response)
    }
}

impl AlkaneResponder for VecTest {
    fn execute(&self) -> Result<CallResponse> {
        Err(anyhow!(
            "This method should not be called directly. Use the declare_alkane macro instead."
        ))
    }
}

declare_alkane! {
    impl AlkaneResponder for VecTest {
        type Message = VecTestMessage;
    }
}

// Compile the VecTest contract to WebAssembly
fn compile_vec_test() -> Vec<u8> {
    // In a real test, this would compile the contract to WebAssembly
    // For this example, we'll just return a placeholder
    vec![]
}

#[wasm_bindgen_test]
fn test_vec_abi() -> Result<()> {
    clear();

    let context = Arc::new(Mutex::new(AlkanesRuntimeContext::default()));
    let contract_bytes = compile_vec_test();

    // Create a new instance of the contract
    let mut instance = AlkanesInstance::from_alkane(context, Arc::new(contract_bytes), 100000000)?;

    // Call the __meta function to get the ABI
    let abi_bytes = instance.call_meta()?;

    // Convert the ABI bytes to a string and parse as JSON
    let abi_string = String::from_utf8(abi_bytes.clone())?;
    let abi_json: serde_json::Value = serde_json::from_slice(&abi_bytes)?;

    // Print the ABI for debugging
    println!("VecTest ABI: {}", abi_string);

    // Verify the contract name
    assert_eq!(abi_json["contract"], "VecTest");

    // Verify that methods array exists
    assert!(abi_json["methods"].is_array());
    let methods = abi_json["methods"].as_array().unwrap();

    // Verify the ProcessNumbers method
    let process_numbers = methods
        .iter()
        .find(|m| m["name"] == "process_numbers")
        .unwrap();
    assert_eq!(process_numbers["opcode"].as_u64().unwrap(), 1);
    assert_eq!(
        process_numbers["params"][0]["type"].as_str().unwrap(),
        "Vec<u128>"
    );

    // Verify the ProcessStrings method
    let process_strings = methods
        .iter()
        .find(|m| m["name"] == "process_strings")
        .unwrap();
    assert_eq!(process_strings["opcode"].as_u64().unwrap(), 2);
    assert_eq!(
        process_strings["params"][0]["type"].as_str().unwrap(),
        "Vec<String>"
    );

    // Verify the GetNumbers method
    let get_numbers = methods.iter().find(|m| m["name"] == "get_numbers").unwrap();
    assert_eq!(get_numbers["opcode"].as_u64().unwrap(), 3);
    assert_eq!(get_numbers["returns"].as_str().unwrap(), "Vec<u128>");

    // Verify the GetStrings method
    let get_strings = methods.iter().find(|m| m["name"] == "get_strings").unwrap();
    assert_eq!(get_strings["opcode"].as_u64().unwrap(), 4);
    assert_eq!(get_strings["returns"].as_str().unwrap(), "Vec<String>");

    // Verify the ProcessNestedVec method
    let process_nested_vec = methods
        .iter()
        .find(|m| m["name"] == "process_nested_vec")
        .unwrap();
    assert_eq!(process_nested_vec["opcode"].as_u64().unwrap(), 5);
    assert_eq!(
        process_nested_vec["params"][0]["type"].as_str().unwrap(),
        "Vec<Vec<u128>>"
    );

    Ok(())
}

#[wasm_bindgen_test]
fn test_vec_inputs() -> Result<()> {
    clear();

    // Create a test message with a vector of u128 values
    let message = VecTestMessage::ProcessNumbers {
        numbers: vec![10u128, 20u128, 30u128, 40u128],
    };

    // Convert the message to opcode and inputs
    let opcode = 1u128; // ProcessNumbers opcode
    let mut inputs = Vec::new();

    // Add the length of the vector
    inputs.push(4u128); // 4 elements

    // Add the elements
    inputs.push(10u128);
    inputs.push(20u128);
    inputs.push(30u128);
    inputs.push(40u128);

    // Parse the message from opcode and inputs
    let parsed_message = VecTestMessage::from_opcode(opcode, inputs.clone())?;

    // Verify the parsed message matches the original
    match parsed_message {
        VecTestMessage::ProcessNumbers { numbers } => {
            assert_eq!(numbers.len(), 4);
            assert_eq!(numbers[0], 10u128);
            assert_eq!(numbers[1], 20u128);
            assert_eq!(numbers[2], 30u128);
            assert_eq!(numbers[3], 40u128);
        }
        _ => panic!("Parsed message has incorrect variant"),
    }

    // Test with a vector of strings
    let opcode = 2u128; // ProcessStrings opcode
    let mut inputs = Vec::new();

    // Add the length of the vector
    inputs.push(2u128); // 2 elements

    // Add the first string "hello" with null terminator
    let hello_bytes = "hello\0".as_bytes();
    let mut hello_u128 = 0u128;
    for (i, &byte) in hello_bytes.iter().enumerate() {
        hello_u128 |= (byte as u128) << (i * 8);
    }
    inputs.push(hello_u128);

    // Add the second string "world" with null terminator
    let world_bytes = "world\0".as_bytes();
    let mut world_u128 = 0u128;
    for (i, &byte) in world_bytes.iter().enumerate() {
        world_u128 |= (byte as u128) << (i * 8);
    }
    inputs.push(world_u128);

    // Parse the message from opcode and inputs
    let parsed_message = VecTestMessage::from_opcode(opcode, inputs.clone())?;

    // Verify the parsed message
    match parsed_message {
        VecTestMessage::ProcessStrings { strings } => {
            assert_eq!(strings.len(), 2);
            assert_eq!(strings[0], "hello");
            assert_eq!(strings[1], "world");
        }
        _ => panic!("Parsed message has incorrect variant"),
    }

    // Test with a nested vector
    let opcode = 5u128; // ProcessNestedVec opcode
    let mut inputs = Vec::new();

    // Add the length of the outer vector
    inputs.push(2u128); // 2 inner vectors

    // Add the first inner vector
    inputs.push(3u128); // Length of first inner vector
    inputs.push(1u128); // Elements of first inner vector
    inputs.push(2u128);
    inputs.push(3u128);

    // Add the second inner vector
    inputs.push(2u128); // Length of second inner vector
    inputs.push(4u128); // Elements of second inner vector
    inputs.push(5u128);

    // Parse the message from opcode and inputs
    let parsed_message = VecTestMessage::from_opcode(opcode, inputs.clone())?;

    // Verify the parsed message
    match parsed_message {
        VecTestMessage::ProcessNestedVec { nested } => {
            assert_eq!(nested.len(), 2);
            assert_eq!(nested[0], vec![1u128, 2u128, 3u128]);
            assert_eq!(nested[1], vec![4u128, 5u128]);
        }
        _ => panic!("Parsed message has incorrect variant"),
    }

    Ok(())
}
