use futures::stream::Stream;
use std::pin::Pin;
use tokio::sync::mpsc;

use crate::{error::Result, result::RunResult, stream_events::StreamEvent};

/// Result from a streamed agent run
pub struct StreamedRunResult {
    /// Channel receiver for streaming events
    receiver: mpsc::UnboundedReceiver<StreamEvent>,
    /// Final result (populated when stream completes)
    final_result: Option<RunResult>,
}

impl StreamedRunResult {
    /// Create a new streamed run result
    pub(crate) fn new(receiver: mpsc::UnboundedReceiver<StreamEvent>) -> Self {
        Self {
            receiver,
            final_result: None,
        }
    }

    /// Get a stream of events
    pub fn stream_events(self) -> Pin<Box<dyn Stream<Item = StreamEvent> + Send>> {
        Box::pin(async_stream::stream! {
            let mut receiver = self.receiver;
            while let Some(event) = receiver.recv().await {
                yield event;
            }
        })
    }

    /// Wait for the final result
    pub async fn final_result(mut self) -> Result<RunResult> {
        // Drain all events
        while self.receiver.recv().await.is_some() {}

        self.final_result.ok_or_else(|| {
            crate::error::AgentError::ConfigError("No final result available".to_string())
        })
    }

    /// Set the final result (internal use)
    pub(crate) fn set_final_result(&mut self, result: RunResult) {
        self.final_result = Some(result);
    }
}
