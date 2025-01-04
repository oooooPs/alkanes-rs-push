use crate::{
    message::AlkaneMessageContext,
    vm::{AlkanesInstance, AlkanesState},
};
use alkanes_support::utils::overflow_error;
use anyhow::Result;
use bitcoin::{Block, Transaction, Witness};
use ordinals::{Artifact, Runestone};
use protorune::message::MessageContext;
use protorune_support::protostone::Protostone;
use protorune_support::utils::decode_varint_list;
use std::io::Cursor;
use wasmi::*;

#[allow(unused_imports)]
use {
    metashrew::{println, stdio::stdout},
    std::fmt::Write,
};

pub trait VirtualFuelBytes {
    fn vfsize(&self) -> u64;
}

impl VirtualFuelBytes for Transaction {
    fn vfsize(&self) -> u64 {
        if let Some(Artifact::Runestone(ref runestone)) = Runestone::decipher(&self) {
            if let Ok(protostones) = Protostone::from_runestone(runestone) {
                let cellpacks = protostones
                    .iter()
                    .filter_map(|v| {
                        if v.protocol_tag == AlkaneMessageContext::protocol_tag() {
                            decode_varint_list(&mut Cursor::new(v.message.clone()))
                                .and_then(|list| {
                                    if list.len() >= 2 {
                                        Ok(Some(list))
                                    } else {
                                        Ok(None)
                                    }
                                })
                                .unwrap_or_else(|_| None)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Vec<u128>>>();
                if cellpacks.len() == 0 {
                    0
                } else if cellpacks
                    .iter()
                    .position(|v| {
                        <&[u128] as TryInto<[u128; 2]>>::try_into(&v[0..2]).unwrap()
                            == [1u128, 0u128]
                            || v[0] == 3u128
                    })
                    .is_some()
                {
                    let mut cloned = self.clone();
                    if cloned.input.len() > 0 {
                        cloned.input[0].witness = Witness::new();
                    }
                    cloned.vsize() as u64
                } else {
                    self.vsize() as u64
                }
            } else {
                0
            }
        } else {
            0
        }
    }
}

impl VirtualFuelBytes for Block {
    fn vfsize(&self) -> u64 {
        self.txdata.iter().fold(0u64, |r, v| r + v.vfsize())
    }
}

//use if regtest
#[cfg(not(any(
    feature = "mainnet",
    feature = "dogecoin",
    feature = "bellscoin",
    feature = "fractal",
    feature = "luckycoin"
)))]
const TOTAL_FUEL: u64 = 100_000_000;
#[cfg(feature = "mainnet")]
const TOTAL_FUEL: u64 = 100_000_000;
#[cfg(feature = "dogecoin")]
const TOTAL_FUEL: u64 = 60_000_000;
#[cfg(feature = "fractal")]
const TOTAL_FUEL: u64 = 50_000_000;
#[cfg(feature = "luckycoin")]
const TOTAL_FUEL: u64 = 50_000_000;
#[cfg(feature = "bellscoin")]
const TOTAL_FUEL: u64 = 50_000_000;

#[derive(Default, Clone, Debug)]
pub struct FuelTank {
    pub current_txindex: u32,
    pub size: u64,
    pub txsize: u64,
    pub block_fuel: u64,
    pub transaction_fuel: u64,
    pub block_metered_fuel: u64,
}

static mut _FUEL_TANK: Option<FuelTank> = None;

#[allow(static_mut_refs)]
impl FuelTank {
    pub fn should_advance(txindex: u32) -> bool {
        unsafe { _FUEL_TANK.as_ref().unwrap().current_txindex != txindex }
    }
    pub fn is_top() -> bool {
        unsafe { _FUEL_TANK.as_ref().unwrap().current_txindex == u32::MAX }
    }
    pub fn initialize(block: &Block) {
        unsafe {
            _FUEL_TANK = Some(FuelTank {
                current_txindex: u32::MAX,
                txsize: 0,
                size: block.vfsize(),
                block_fuel: TOTAL_FUEL,
                transaction_fuel: 0,
                block_metered_fuel: 0,
            });
        }
    }
    pub fn fuel_transaction(txsize: u64, txindex: u32) {
        unsafe {
            let tank: &'static mut FuelTank = _FUEL_TANK.as_mut().unwrap();
            tank.current_txindex = txindex;
            tank.block_metered_fuel = tank.block_fuel * txsize / tank.size;
            tank.transaction_fuel = std::cmp::max(MINIMUM_FUEL, tank.block_metered_fuel);
            tank.block_fuel =
                tank.block_fuel - std::cmp::min(tank.block_fuel, tank.block_metered_fuel);
            tank.txsize = txsize;
        }
    }
    pub fn refuel_block() {
        unsafe {
            let tank: &'static mut FuelTank = _FUEL_TANK.as_mut().unwrap();
            tank.block_fuel = tank.block_fuel + tank.block_metered_fuel;
            tank.size = tank.size - tank.txsize;
        }
    }
    pub fn consume_fuel(n: u64) -> Result<()> {
        unsafe {
            let tank: &'static mut FuelTank = _FUEL_TANK.as_mut().unwrap();
            tank.transaction_fuel = overflow_error(tank.transaction_fuel.checked_sub(n))?;
            tank.block_metered_fuel =
                overflow_error(tank.block_metered_fuel.checked_sub(n)).unwrap_or_else(|_| 0);
            Ok(())
        }
    }
    pub fn drain_fuel() {
        unsafe {
            let transaction_fuel = _FUEL_TANK.as_ref().unwrap().block_metered_fuel;
            let tank: &'static mut FuelTank = _FUEL_TANK.as_mut().unwrap();
            tank.block_fuel = tank.block_fuel - std::cmp::min(tank.block_fuel, transaction_fuel);
            tank.transaction_fuel = 0;
            tank.block_metered_fuel = 0;
        }
    }
    pub fn start_fuel() -> u64 {
        unsafe { _FUEL_TANK.as_ref().unwrap().transaction_fuel }
    }
}

pub const MINIMUM_FUEL: u64 = 90_000;
pub const FUEL_PER_VBYTE: u64 = 150;
pub const FUEL_PER_REQUEST_BYTE: u64 = 1;
pub const FUEL_PER_LOAD_BYTE: u64 = 2;
pub const FUEL_PER_STORE_BYTE: u64 = 8;
pub const FUEL_SEQUENCE: u64 = 5;
pub const FUEL_FUEL: u64 = 5;
pub const FUEL_EXTCALL: u64 = 500;
pub const FUEL_HEIGHT: u64 = 10;
pub const FUEL_BALANCE: u64 = 10;
pub const FUEL_EXTCALL_DEPLOY: u64 = 10_000;

pub trait Fuelable {
    fn consume_fuel(&mut self, n: u64) -> Result<()>;
}

impl<'a> Fuelable for Caller<'_, AlkanesState> {
    fn consume_fuel(&mut self, n: u64) -> Result<()> {
        overflow_error((self.get_fuel().unwrap() as u64).checked_sub(n))?;
        Ok(())
    }
}

impl Fuelable for AlkanesInstance {
    fn consume_fuel(&mut self, n: u64) -> Result<()> {
        overflow_error((self.store.get_fuel().unwrap() as u64).checked_sub(n))?;
        Ok(())
    }
}

pub fn consume_fuel<'a>(caller: &mut Caller<'_, AlkanesState>, n: u64) -> Result<()> {
    caller.consume_fuel(n)
}

pub fn compute_extcall_fuel(savecount: u64) -> Result<u64> {
    let save_fuel = overflow_error(FUEL_PER_STORE_BYTE.checked_mul(savecount))?;
    overflow_error::<u64>(FUEL_EXTCALL.checked_add(save_fuel))
}
