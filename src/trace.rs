#[allow(unused_imports)]
use {
  metashrew::{println, stdio::{stdout}},
  std::fmt::{Write}
};
use metashrew_support::utils::{consensus_encode};
use metashrew_support::index_pointer::{KeyValuePointer};
use alkanes_support::trace::{Trace};
use crate::tables::{TRACES, TRACES_BY_HEIGHT};
use alkanes_support::proto;
use anyhow::{Result};
use bitcoin::{OutPoint};
use protobuf::{Message};
use std::sync::{Arc};

pub fn save_trace(outpoint: &OutPoint, height: u64, trace: Trace) -> Result<()> {
  println!("save trace: {:?}", outpoint);
/*
  println!("trace dump: {:?}", trace.0.lock().unwrap().clone());
*/
  let buffer: Vec<u8> = consensus_encode::<OutPoint>(outpoint)?;
  TRACES.select(&buffer).set(Arc::<Vec<u8>>::new(<Trace as Into<proto::alkanes::AlkanesTrace>>::into(trace).write_to_bytes()?));
  TRACES_BY_HEIGHT.select_value(height).append(Arc::new(buffer));
  Ok(())
}
