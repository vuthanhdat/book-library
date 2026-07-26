use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct OperationId(Uuid);

impl OperationId {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Events are facts and therefore use stable, past-tense names at construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EventEnvelope<Payload> {
    pub(crate) name: &'static str,
    pub(crate) operation_id: OperationId,
    pub(crate) payload: Payload,
}

impl<Payload> EventEnvelope<Payload> {
    pub(crate) fn fact(name: &'static str, operation_id: OperationId, payload: Payload) -> Self {
        debug_assert!(
            name.ends_with("_started")
                || name.ends_with("_progressed")
                || name.ends_with("_completed")
                || name.ends_with("_failed")
                || name.ends_with("_cancelled"),
            "event names must describe stable past-tense facts"
        );
        Self {
            name,
            operation_id,
            payload,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloned_cancellation_tokens_share_state() {
        let first = CancellationToken::default();
        let second = first.clone();

        first.cancel();

        assert!(second.is_cancelled());
    }

    #[test]
    fn event_envelope_keeps_operation_correlation() {
        let operation_id = OperationId::new();
        let event = EventEnvelope::fact("scan_started", operation_id, 42);

        assert_eq!(event.name, "scan_started");
        assert_eq!(event.operation_id, operation_id);
        assert_eq!(event.payload, 42);
    }
}
