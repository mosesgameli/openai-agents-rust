use futures::stream::Stream;
use std::pin::Pin;
use tokio::sync::mpsc;

use crate::{error::Result, result::RunResult, stream_events::StreamEvent};

use std::sync::Arc;

/// Shared state for StreamedRunResult
pub(crate) struct SharedState {
    pub(crate) final_result: Option<RunResult>,
}

/// Result from a streamed agent run
#[derive(Clone)]
pub struct StreamedRunResult {
    /// Channel receiver for streaming events
    receiver: Arc<tokio::sync::Mutex<Option<mpsc::UnboundedReceiver<StreamEvent>>>>,
    /// Shared state
    shared_state: Arc<std::sync::Mutex<SharedState>>,
}

impl StreamedRunResult {
    /// Create a new streamed run result
    pub(crate) fn new(
        receiver: mpsc::UnboundedReceiver<StreamEvent>,
    ) -> (Self, Arc<std::sync::Mutex<SharedState>>) {
        let shared_state = Arc::new(std::sync::Mutex::new(SharedState { final_result: None }));
        (
            Self {
                receiver: Arc::new(tokio::sync::Mutex::new(Some(receiver))),
                shared_state: shared_state.clone(),
            },
            shared_state,
        )
    }

    /// Get a stream of events
    pub fn stream_events(&self) -> Pin<Box<dyn Stream<Item = StreamEvent> + Send>> {
        let receiver_arc = self.receiver.clone();
        Box::pin(async_stream::stream! {
            let mut opt_receiver = receiver_arc.lock().await;
            if let Some(mut receiver) = opt_receiver.take() {
                while let Some(event) = receiver.recv().await {
                    yield event;
                }
            }
        })
    }

    /// Wait for the final result
    pub async fn final_result(&self) -> Result<RunResult> {
        // Drain all events if not already done
        {
            let mut opt_receiver = self.receiver.lock().await;
            if let Some(mut receiver) = opt_receiver.take() {
                while receiver.recv().await.is_some() {}
            }
        }

        let state = self.shared_state.lock().unwrap();
        state.final_result.clone().ok_or_else(|| {
            crate::error::AgentError::ConfigError("No final result available".to_string())
        })
    }

    /// Get the final output directly (convenience)
    pub async fn final_output(&self) -> Result<String> {
        let result = self.final_result().await?;
        Ok(result.final_output().to_string())
    }
}
