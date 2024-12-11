use crate::view::{simulate_parcel, parcel_from_protobuf};
use alkanes_support::proto;
use crate::indexer::{configure_network};
use bitcoin::{Block};
#[allow(unused_imports)]
use metashrew::{
    flush, input, println,
    stdio::{stdout, Write},
};
#[allow(unused_imports)]
use metashrew_support::block::AuxpowBlock;
use metashrew_support::utils::{consensus_decode, consume_sized_int, consume_to_end};
use metashrew_support::compat::{export_bytes, to_arraybuffer_layout, to_passback_ptr};
use protobuf::{Message, MessageField};
use std::io::Cursor;
pub mod message;
pub mod indexer;
pub mod network;
pub mod precompiled;
#[cfg(any(test, feature = "test-utils"))]
pub mod tests;
pub mod utils;
pub mod view;
pub mod vm;
pub mod block;
use crate::indexer::{index_block};


#[no_mangle]
pub fn simulate() -> i32 {
    configure_network();
    let data = input();
    let _height = u32::from_le_bytes((&data[0..4]).try_into().unwrap());
    let reader = &data[4..];
    let mut result: proto::alkanes::SimulateResponse = proto::alkanes::SimulateResponse::new();
    match simulate_parcel(&parcel_from_protobuf(
        proto::alkanes::MessageContextParcel::parse_from_bytes(reader).unwrap(),
    ), u64::MAX) {
        Ok((response, gas_used)) => {
            result.execution = MessageField::some(response.into());
            result.gas_used = gas_used;
        }
        Err(e) => {
            result.error = e.to_string();
        }
    }
    to_passback_ptr(&mut to_arraybuffer_layout::<&[u8]>(
        result.write_to_bytes().unwrap().as_ref(),
    ))
}

#[no_mangle]
pub fn runesbyaddress() -> i32 {
    configure_network();
    let mut data: Cursor<Vec<u8>> = Cursor::new(input());
    let _height = consume_sized_int::<u32>(&mut data).unwrap();
    let result: protorune_support::proto::protorune::WalletResponse = protorune::view::runes_by_address(&consume_to_end(&mut data).unwrap()).unwrap_or_else(|_| protorune_support::proto::protorune::WalletResponse::new());
    to_passback_ptr(&mut to_arraybuffer_layout::<&[u8]>(result.write_to_bytes().unwrap().as_ref()))
}

#[no_mangle]
pub fn protorunesbyaddress() -> i32 {
    configure_network();
    let mut data: Cursor<Vec<u8>> = Cursor::new(input());
    let _height = consume_sized_int::<u32>(&mut data).unwrap();
    let result: protorune_support::proto::protorune::WalletResponse = view::protorunes_by_address(&consume_to_end(&mut data).unwrap()).unwrap_or_else(|_| protorune_support::proto::protorune::WalletResponse::new());
    export_bytes(result.write_to_bytes().unwrap())
}


#[no_mangle]
pub fn protorunesbyoutpoint() -> i32 {
    configure_network();
    let mut data: Cursor<Vec<u8>> = Cursor::new(input());
    let _height = consume_sized_int::<u32>(&mut data).unwrap();
    let result: protorune_support::proto::protorune::OutpointResponse = view::protorunes_by_outpoint(&consume_to_end(&mut data).unwrap()).unwrap_or_else(|_| protorune_support::proto::protorune::OutpointResponse::new());
  
    export_bytes(result.write_to_bytes().unwrap())
}

#[no_mangle]
pub fn runesbyheight() -> i32 {
    configure_network();
    let mut data: Cursor<Vec<u8>> = Cursor::new(input());
    let _height = consume_sized_int::<u32>(&mut data).unwrap();
    let result: protorune_support::proto::protorune::RunesResponse = protorune::view::runes_by_height(&consume_to_end(&mut data).unwrap()).unwrap_or_else(|_| protorune_support::proto::protorune::RunesResponse::new());
    let buffer: Vec<u8> = result.write_to_bytes().unwrap();
    to_passback_ptr(&mut to_arraybuffer_layout::<&[u8]>(buffer.as_ref()))
}


// #[no_mangle]
// pub fn alkane_balance_sheet() -> i32 {
//     let data = input();
//     let _height = u32::from_le_bytes((&data[0..4]).try_into().unwrap());
//     let reader = &data[4..];
//     let mut result: proto::alkanes::SimulateResponse = proto::alkanes::SimulateResponse::new();
//     let (response, gas_used) = alkane_inventory(
//         &proto::alkanes::MessageContextParcel::parse_from_bytes(reader).unwrap().into()
//     ).unwrap();
//     result.execution = MessageField::some(response.into());
//     result.gas_used = gas_used;
//     to_passback_ptr(&mut to_arraybuffer_layout::<&[u8]>(result.write_to_bytes().unwrap().as_ref()))
// }
//
//

#[no_mangle]
pub fn _start() {
    let data = input();
    let height = u32::from_le_bytes((&data[0..4]).try_into().unwrap());
    let reader = &data[4..];
    #[cfg(any(feature = "dogecoin", feature = "luckycoin", feature = "bellscoin"))]
    let block: Block = AuxpowBlock::parse(&mut Cursor::<Vec<u8>>::new(reader.to_vec()))
        .unwrap()
        .to_consensus();
    #[cfg(not(any(feature = "dogecoin", feature = "luckycoin", feature = "bellscoin")))]
    let block: Block = consensus_decode::<Block>(&mut Cursor::<Vec<u8>>::new(reader.to_vec())).unwrap();
    index_block(&block, height).unwrap();
    flush();
}
