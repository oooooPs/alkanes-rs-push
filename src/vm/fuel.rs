use crate::vm::{AlkanesInstance, AlkanesState};
use alkanes_support::utils::overflow_error;
use anyhow::Result;
use wasmi::*;

pub fn fuelsize(block: &Block) {
    let mut size: u64 = 0;
    for tx in &block.txdata {
        if let Some(Artifact::Runestone(ref runestone)) = Runestone::decipher(tx) {
            if let Ok(protostones) = Protostone::from_runestone(runestone) {
                for protostone in protostones {
                    if protostone.protocol_tag == AlkaneMessageContext::protocol_tag()
                        && protostone.message.len() != 0
                    {
                        size = size + tx.total_size();
                    }
                }
            }
        }
    }
    size
}

//use if regtest
#[cfg(not(all(feature="mainnet", feature="dogecoin", feature="bellscoin", feature="fractal", feature="luckycoin")))]
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
  pub size: usize,
  pub block_fuel: u64,
  pub transaction_fuel: u64
}

static mut _FUEL_TANK: Option<FuelTank> = None;

impl FuelTank {
  pub fn should_advance(txindex: u32) -> bool {
    unsafe {
      _FUEL_TANK.as_ref().unwrap().current_txindex != txindex
    }
  }
  pub fn is_top() -> bool {
    unsafe {
      _FUEL_TANK.as_ref().unwrap().current_txindex == u32::MAX
    }
  }
  pub fn initialize(block: &Block) {
    unsafe {
      _FUEL_TANK = Some(FuelTank {
        current_txindex: u32::MAX,
        size: fuelsize(block),
        block_fuel: TOTAL_FUEL,
        transaction_fuel: 0
      });
    }
  }
  pub fn fuel_transaction(txsize: usize, txindex: usize) -> {
    unsafe {
      let mut tank: &'static mut FuelTank = _FUEL_TANK.as_mut().unwrap();
      tank.current_txindex = txindex;
      tank.transaction_fuel = std::cmp::min(tank.block_fuel*txsize/tank.size, MINIMUM_FUEL);
      tank.txsize = txsize;
    }
  }
  pub fn refuel_block() -> {
    unsafe {
      let mut tank: &'static mut FuelTank = _FUEL_TANK.as_mut().unwrap();
      let start_fuel = tank.block_fuel*tank.txsize/tank.size;
      tank.block_fuel = tank.block_fuel + tank.transaction_fuel - start_fuel;
      tank.size = tank.size - tank.txsize;
    }
  }
  pub fn burn(v: u64) -> Result<()> {
    unsafe {
      tank.as_mut().unwrap().transaction_fuel >= v
    }
[ }
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

pub fn start_fuel() -> u64 {
    std::cmp::max(TOTAL_FUEL / std::cmp::max(1, unsafe { MESSAGE_COUNT }), MINIMUM_FUEL)
}

pub fn compute_extcall_fuel(savecount: u64) -> Result<u64> {
    let save_fuel = overflow_error(FUEL_PER_STORE_BYTE.checked_mul(savecount))?;
    overflow_error::<u64>(FUEL_EXTCALL.checked_add(save_fuel))
}
