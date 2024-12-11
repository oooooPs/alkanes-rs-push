use alkanes_support::context::{Context};
use alkanes_support::id::{AlkaneId};
use metashrew_support::index_pointer::{KeyValuePointer};
use metashrew::index_pointer::{IndexPointer};
use metashrew_support::utils::{consensus_encode};
use bitcoin::{OutPoint};

#[derive(Debug, Clone, Default)]
pub struct TraceContext {
  inner: Context,
  target: AlkaneId,
  fuel: u64
}

#[derive(Debug, Clone, Default)]
pub struct TraceResponse {
  pub inner: ExtendedCallResponse,
  pub fuel_used: u64
}

#[derive(Debug, Clone)]
enum TraceEvent {
  EnterDelegatecall(TraceContext),
  EnterStaticcall(TraceContext),
  EnterCall(TraceContext),
  RevertContext(TraceResponse),
  ReturnContext(TraceResponse)
}

impl Into<proto::alkanes::AlkanesTrace> for Vec<TraceEvent> {
  fn into(self) -> proto::alkanes::AlkanesTrace {
    let mut result = proto::alkanes::AlkanesTrace::new();
    result.events = self.into_iter().map(|v| v.into()).collect::<Vec<proto::alkanes::AlkanesTraceEvent>>();
    result
  }
}

fn save_trace(txid: Txid, vout: u32, trace: Vec<TraceEvent>) -> Result<()> {
  let outpoint: Vec<u8> = consensus_encode::<OutPoint>(&OutPoint {
    txid,
    vout
  })?;
  IndexPointer::from_keyword("/traces/").select(&outpoint).set(Arc::new(trace.into().write_to_bytes()?))
  IndexPointer::from_keyword("/traces/byheight/").select_value(height).append(&outpoint);
  Ok(())
}
