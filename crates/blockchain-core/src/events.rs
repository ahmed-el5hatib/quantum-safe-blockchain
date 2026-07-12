use std::fmt;

use crate::types::Event;
use crate::CoreResult;

pub trait EventBus: Send + Sync + fmt::Debug {
    fn publish(&self, event: Event) -> CoreResult<()>;
    fn subscribe(&self, handler: EventHandler) -> CoreResult<()>;
}

pub type EventHandler = fn(event: Event);
