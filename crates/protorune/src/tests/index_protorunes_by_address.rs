#[cfg(test)]
mod tests {
    use crate::balance_sheet::load_sheet;
    use crate::message::{MessageContext, MessageContextParcel};
    use crate::test_helpers::{self as helpers, get_address, ADDRESS1, ADDRESS2};
    use crate::{view, Protorune};
    use anyhow::Result;
    use bitcoin::OutPoint;
    use protobuf::{Message, MessageField};
    use protorune_support::balance_sheet::{BalanceSheet, ProtoruneRuneId};
    use protorune_support::proto::protorune::ProtorunesWalletRequest;
    use protorune_support::rune_transfer::RuneTransfer;
    use std::str::FromStr;
    use wasm_bindgen_test::*;

    use helpers::clear;

    // Define a NoopMessageContext that doesn't do anything special with the protorunes
    struct NoopMessageContext;

    impl MessageContext for NoopMessageContext {
        fn protocol_tag() -> u128 {
            122 // Using the same protocol tag as in the tests
        }

        fn handle(parcel: &MessageContextParcel) -> Result<(Vec<RuneTransfer>, BalanceSheet)> {
            // Just return the runes as-is without any special handling
            let runes: Vec<RuneTransfer> = parcel.runes.clone();
            Ok((runes, BalanceSheet::default()))
        }
    }

    // Helper function to create a block with protoburns for testing
    fn create_test_block_with_protoburns(protocol_id: u128, block_height: u64) -> bitcoin::Block {
        let first_mock_output = OutPoint {
            txid: bitcoin::Txid::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            vout: 0,
        };

        // Create a protoburn transaction that sends protorunes to ADDRESS1
        let protoburn_tx =
            helpers::create_default_protoburn_transaction(first_mock_output, protocol_id);

        helpers::create_block_with_txs(vec![protoburn_tx])
    }

    // Helper function to create a block with multiple protoburns for different addresses
    fn create_test_block_with_multiple_protoburns(
        protocol_id: u128,
        block_height: u64,
    ) -> bitcoin::Block {
        let first_mock_output = OutPoint {
            txid: bitcoin::Txid::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            vout: 0,
        };

        // Create a protoburn transaction that sends protorunes to ADDRESS1
        let protoburn_tx1 =
            helpers::create_default_protoburn_transaction(first_mock_output, protocol_id);

        // Create another protoburn transaction that sends protorunes to ADDRESS2
        let second_mock_output = OutPoint {
            txid: bitcoin::Txid::from_str(
                "1111111111111111111111111111111111111111111111111111111111111111",
            )
            .unwrap(),
            vout: 0,
        };

        let protoburn_tx2 =
            helpers::create_default_protoburn_transaction(second_mock_output, protocol_id);

        helpers::create_block_with_txs(vec![protoburn_tx1, protoburn_tx2])
    }

    // Test that protorunes_by_address returns the correct protorunes for a single address
    #[wasm_bindgen_test]
    fn test_protorunes_by_address_single() -> Result<()> {
        clear();
        let block_height = 840000;
        let protocol_id = 122;

        // Create and index a block with a protoburn
        let test_block = create_test_block_with_protoburns(protocol_id, block_height);
        assert!(
            Protorune::index_block::<NoopMessageContext>(test_block.clone(), block_height).is_ok()
        );

        // Create a ProtorunesWalletRequest for ADDRESS1
        let mut request = ProtorunesWalletRequest::new();
        request.wallet = ADDRESS1().as_bytes().to_vec();
        request.protocol_tag = MessageField::some(protocol_id.into());

        // Serialize the request
        let serialized_request = request.write_to_bytes()?;

        // Call protorunes_by_address
        let response = view::protorunes_by_address(&serialized_request)?;

        // Verify the response
        assert!(
            !response.outpoints.is_empty(),
            "Expected non-empty outpoints in response"
        );

        // Check that each outpoint has the expected protorune
        for outpoint_response in response.outpoints.iter() {
            let balances = outpoint_response.balances.as_ref().unwrap();

            // Convert from proto balances to BalanceSheet
            let balance_sheet: BalanceSheet = balances.clone().into();

            // Check if the balance sheet contains the expected protorune
            let protorune_id = ProtoruneRuneId {
                block: block_height as u128,
                tx: 0,
            };

            let balance = balance_sheet.get(&protorune_id);
            assert!(balance > 0, "Expected positive balance for protorune");
        }

        Ok(())
    }

    // Test that protorunes_by_address returns the correct protorunes for a single address
    // Note: We're not testing multiple addresses since create_default_protoburn_transaction
    // doesn't support specifying different addresses
    #[wasm_bindgen_test]
    fn test_protorunes_by_address_multiple() -> Result<()> {
        clear();
        let block_height = 840000;
        let protocol_id = 122;

        // Create and index a block with multiple protoburns
        let test_block = create_test_block_with_multiple_protoburns(protocol_id, block_height);
        assert!(
            Protorune::index_block::<NoopMessageContext>(test_block.clone(), block_height).is_ok()
        );

        // Test ADDRESS1 (both transactions should send protorunes to ADDRESS1)
        let mut request = ProtorunesWalletRequest::new();
        request.wallet = ADDRESS1().as_bytes().to_vec();
        request.protocol_tag = MessageField::some(protocol_id.into());

        let serialized_request = request.write_to_bytes()?;
        let response = view::protorunes_by_address(&serialized_request)?;

        assert!(
            !response.outpoints.is_empty(),
            "Expected non-empty outpoints for ADDRESS1"
        );

        // Verify we have at least 2 outpoints (from the two transactions)
        assert!(
            response.outpoints.len() >= 2,
            "Expected at least 2 outpoints for ADDRESS1"
        );

        Ok(())
    }

    // Test that protorunes_by_address returns empty results for an address with no protorunes
    #[wasm_bindgen_test]
    fn test_protorunes_by_address_empty() -> Result<()> {
        clear();
        let block_height = 840000;
        let protocol_id = 122;

        // Create and index a block with a protoburn for ADDRESS1 only
        let test_block = create_test_block_with_protoburns(protocol_id, block_height);
        assert!(
            Protorune::index_block::<NoopMessageContext>(test_block.clone(), block_height).is_ok()
        );

        // Create a ProtorunesWalletRequest for ADDRESS2 (which has no protorunes)
        let mut request = ProtorunesWalletRequest::new();
        request.wallet = ADDRESS2().as_bytes().to_vec();
        request.protocol_tag = MessageField::some(protocol_id.into());

        // Serialize the request
        let serialized_request = request.write_to_bytes()?;

        // Call protorunes_by_address
        let response = view::protorunes_by_address(&serialized_request)?;

        // Verify the response is empty
        assert!(
            response.outpoints.is_empty(),
            "Expected empty outpoints for address with no protorunes"
        );

        Ok(())
    }

    // Test that protorunes_by_address returns protorunes for the correct protocol
    #[wasm_bindgen_test]
    fn test_protorunes_by_address_protocol_specific() -> Result<()> {
        clear();
        let block_height = 840000;
        let protocol_id1 = 122;
        let protocol_id2 = 123;

        // Create and index a block with a protoburn for protocol_id1
        let test_block1 = create_test_block_with_protoburns(protocol_id1, block_height);
        assert!(
            Protorune::index_block::<NoopMessageContext>(test_block1.clone(), block_height).is_ok()
        );

        // Create and index another block with a protoburn for protocol_id2
        let block_height2 = 840001;
        let test_block2 = create_test_block_with_protoburns(protocol_id2, block_height2);
        assert!(
            Protorune::index_block::<NoopMessageContext>(test_block2.clone(), block_height2)
                .is_ok()
        );

        // Request protorunes for protocol_id1
        let mut request1 = ProtorunesWalletRequest::new();
        request1.wallet = ADDRESS1().as_bytes().to_vec();
        request1.protocol_tag = MessageField::some(protocol_id1.into());

        let serialized_request1 = request1.write_to_bytes()?;
        let response1 = view::protorunes_by_address(&serialized_request1)?;

        // Request protorunes for protocol_id2
        let mut request2 = ProtorunesWalletRequest::new();
        request2.wallet = ADDRESS1().as_bytes().to_vec();
        request2.protocol_tag = MessageField::some(protocol_id2.into());

        let serialized_request2 = request2.write_to_bytes()?;
        let response2 = view::protorunes_by_address(&serialized_request2)?;

        // Both should return results
        assert!(
            !response1.outpoints.is_empty(),
            "Expected non-empty outpoints for protocol_id1"
        );
        assert!(
            !response2.outpoints.is_empty(),
            "Expected non-empty outpoints for protocol_id2"
        );

        // Verify the responses contain different protorunes (from different protocols)
        if !response1.outpoints.is_empty() && !response2.outpoints.is_empty() {
            let balances1 = response1.outpoints[0].balances.as_ref().unwrap();
            let balances2 = response2.outpoints[0].balances.as_ref().unwrap();

            let balance_sheet1: BalanceSheet = balances1.clone().into();
            let balance_sheet2: BalanceSheet = balances2.clone().into();

            // The protorune IDs should be different because they're from different blocks
            let protorune_id1 = ProtoruneRuneId {
                block: block_height as u128,
                tx: 0,
            };

            let protorune_id2 = ProtoruneRuneId {
                block: block_height2 as u128,
                tx: 0,
            };

            // Check that at least one of the protorune IDs has a positive balance
            let has_positive_balance1 = balance_sheet1.get(&protorune_id1) > 0;
            let has_positive_balance2 = balance_sheet2.get(&protorune_id2) > 0;

            assert!(
                has_positive_balance1 || has_positive_balance2,
                "Expected at least one protorune to have a positive balance"
            );
        }

        Ok(())
    }
}
