use crate::context::{Context};
use crate::id::{AlkaneId};
use crate::response::{ExtendedCallResponse};
use crate::proto;
use protobuf::{MessageField};
use std::sync::{Arc, Mutex};


#[derive(Debug, Clone, Default)]
pub struct TraceContext {
  pub inner: Context,
  pub target: AlkaneId,
  pub fuel: u64
}

#[derive(Debug, Clone, Default)]
pub struct TraceResponse {
  pub inner: ExtendedCallResponse,
  pub fuel_used: u64
}

#[derive(Debug, Clone)]
pub enum TraceEvent {
  EnterDelegatecall(TraceContext),
  EnterStaticcall(TraceContext),
  EnterCall(TraceContext),
  RevertContext(TraceResponse),
  ReturnContext(TraceResponse),
  CreateAlkane(AlkaneId)
}

impl Into<TraceResponse> for ExtendedCallResponse {
  fn into(self) -> TraceResponse {
    TraceResponse {
      inner: self,
      fuel_used: 0
    }
  }
}

impl Into<TraceContext> for Context {
  fn into(self) -> TraceContext {
    let target = self.myself.clone();
    TraceContext {
      inner: self,
      target,
      fuel: 0
    }
  }
}

impl Into<proto::alkanes::Context> for Context {
  fn into(self) -> proto::alkanes::Context {
    let mut result = proto::alkanes::Context::new();
    result.myself = MessageField::some(self.myself.into());
    result.caller = MessageField::some(self.caller.into());
    result.vout = self.vout as u32;
    result.incoming_alkanes = self.incoming_alkanes.0.into_iter().map(|v| v.into()).collect::<Vec<proto::alkanes::AlkaneTransfer>>();
    result
  }
}

impl Into<proto::alkanes::AlkanesExitContext> for TraceResponse {
  fn into(self) -> proto::alkanes::AlkanesExitContext {
    let mut result = proto::alkanes::AlkanesExitContext::new();
    result.response = MessageField::some(self.inner.into());
    result
  }
}

impl Into<proto::alkanes::TraceContext> for TraceContext {
  fn into(self) -> proto::alkanes::TraceContext {
    let mut result = proto::alkanes::TraceContext::new();
    result.inner = MessageField::some(self.inner.into());
    result.fuel = self.fuel;
    result
  }
}

impl Into<proto::alkanes::AlkanesEnterContext> for TraceContext {
  fn into(self) -> proto::alkanes::AlkanesEnterContext {
    let mut result = proto::alkanes::AlkanesEnterContext::new();
    result.context = MessageField::some(self.into());
    result
  }
}

impl Into<proto::alkanes::AlkanesTraceEvent> for TraceEvent {
  fn into(self) -> proto::alkanes::AlkanesTraceEvent {
    let mut result = proto::alkanes::AlkanesTraceEvent::new();
    result.event = Some(match self {
      TraceEvent::EnterCall(v) => {
        let mut context: proto::alkanes::AlkanesEnterContext = v.into();
        context.call_type = protobuf::EnumOrUnknown::from_i32(1);
        proto::alkanes::alkanes_trace_event::Event::EnterContext(context)
      },
      TraceEvent::EnterStaticcall(v) => {
        let mut context: proto::alkanes::AlkanesEnterContext = v.into();
        context.call_type = protobuf::EnumOrUnknown::from_i32(3);
        proto::alkanes::alkanes_trace_event::Event::EnterContext(context)
      }
      TraceEvent::EnterDelegatecall(v) => {
        let mut context: proto::alkanes::AlkanesEnterContext = v.into();
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
      TraceEvent::CreateAlkane(v) => {
        let mut creation = proto::alkanes::AlkanesCreate::new();
        creation.new_alkane = MessageField::some(v.into());
        proto::alkanes::alkanes_trace_event::Event::CreateAlkane(creation)
      }
    });
    result
  }
}

#[derive(Debug, Default)]
pub struct Trace(pub Arc<Mutex<Vec<TraceEvent>>>);

impl Trace {
  pub fn clock(&self, event: TraceEvent) {
    self.0.lock().unwrap().push(event);
  }
}

impl Clone for Trace {
  fn clone(&self) -> Self {
    Trace(self.0.clone())
  }
}

impl Into<proto::alkanes::AlkanesTrace> for Vec<TraceEvent> {
  fn into(self) -> proto::alkanes::AlkanesTrace {
    let mut result = proto::alkanes::AlkanesTrace::new();
    result.events = self.into_iter().map(|v| v.into()).collect::<Vec<proto::alkanes::AlkanesTraceEvent>>();
    result
  }
}

impl Into<proto::alkanes::AlkanesTrace> for Trace {
  fn into(self) -> proto::alkanes::AlkanesTrace {
    self.0.lock().unwrap().clone().into()
  }
}
