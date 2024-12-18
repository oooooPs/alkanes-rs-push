use bitcoin::{OutPoint};
use crate::trace::types::{TraceEvent};

#[derive(Clone, Debug, Default)]
pub struct BlockTraceItem {
  pub outpoint: OutPoint,
  pub trace: Vec<TraceEvent>,
}
