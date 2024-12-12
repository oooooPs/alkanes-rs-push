use alkanes_support::context::{Context};
use alkanes_support::id::{AlkaneId};
use alkanes_support::response::{ExtendedCallResponse};
//use metashrew_support::index_pointer::{KeyValuePointer};
//use metashrew::index_pointer::{IndexPointer};
//use metashrew_support::utils::{consensus_encode};
//use bitcoin::{Txid, OutPoint};
use alkanes_support::proto;
use protobuf::{Message, MessageField};


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
/*


pub fn into_context(v: Context) -> proto::alkanes::Context {
  let mut result = proto::alkanes::Context::new();
  result.myself = MessageField::some(v.myself.into());
  result.caller = MessageField::some(v.caller.into());
  result.vout = v.vout as u32;
  result.incoming_alkanes = v.incoming_alkanes.0.into_iter().map(|v| v.into()).collect::<Vec<proto::alkanes::AlkaneTransfer>>();
  result
}
impl Into<proto::alkanes::TraceContext> for TraceContext {
  fn into(self) -> proto::alkanes::TraceContext {
    let mut result = proto::alkanes::TraceContext::new();
    result.inner = MessageField::some(into_context(self.inner));
    result.fuel = self.fuel;
    result
  }
}

impl Into<proto::alkanes::AlkanesTraceEvent> for TraceEvent {
  fn into(self) -> proto::alkanes::AlkanesTraceEvent {
    let mut result = proto::alkanes::AlkanesTraceEvent::new();
    result.event = Some(match self {
      TraceEvent::EnterCall(v) => {
        let mut context: proto::alkanes::TraceContext = v.into();
        context.call_type = protobuf::EnumOrUnknown::from_i32(1);
        proto::alkanes::alkanes_trace_event::Event::EnterContext(context)
      },
      TraceEvent::EnterStaticcall(v) => {
        let mut context: proto::alkanes::TraceContext = v.into();
        context.call_type = protobuf::EnumOrUnknown::from_i32(3);
        proto::alkanes::alkanes_trace_event::Event::EnterContext(context)
      }
      TraceEvent::EnterDelegatecall(v) => {
        let mut context: proto::alkanes::TraceContext = v.into();
        context.call_type = protobuf::EnumOrUnknown::from_i32(2);
        proto::alkanes::alkanes_trace_event::Event::EnterContext(context)
      }
      TraceEvent::ReturnContext(v) => {
        let mut context: proto::alkanes::AlkanesExitContext = v.into();
        context.status = protobuf::EnumOrUnknown::from_i32(0);
        proto::alkanes::alkanes_trace_event::Event::ExitContext(context)
      }
      TraceEvent::RevertContext(v) => {
        let mut context: proto::alkanes::AlkanesExitContext = v.into();
        context.status = protobuf::EnumOrUnknown::from_i32(1);
        proto::alkanes::alkanes_trace_event::Event::ExitContext(context)
      }
    });
    result
  }
}

impl Into<proto::alkanes::AlkanesTrace> for Vec<TraceEvent> {
  fn into(self) -> proto::alkanes::AlkanesTrace {
    let mut result = proto::alkanes::AlkanesTrace::new();
    result.events = self.into_iter().map(|v| v.into()).collect::<Vec<proto::alkanes::AlkanesTraceEvent>>();
    result
  }
}

fn save_trace(txid: Txid, vout: u32, height: u32, trace: Vec<TraceEvent>) -> Result<()> {
  let outpoint: Vec<u8> = consensus_encode::<OutPoint>(&OutPoint {
    txid,
    vout
  })?;
  IndexPointer::from_keyword("/traces/").select(&outpoint).set(Arc::new(trace.into().write_to_bytes()?))
  IndexPointer::from_keyword("/traces/byheight/").select_value(height).append(Arc::new(outpoint));
  Ok(())
}
*/
