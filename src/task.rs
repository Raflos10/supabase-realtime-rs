use std::sync::Arc;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct Cancelled;

#[derive(Clone)]
pub struct Task<T> {
    handle: Arc<JoinHandle<Result<T, Cancelled>>>,
    cancellation_token: CancellationToken,
}

impl<T> Task<T>
where
    T: Send + 'static,
{
    pub fn spawn<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        let cancellation_token = CancellationToken::new();
        let cancellation_token_clone = cancellation_token.clone();
        let handle = tokio::spawn(async move {
            tokio::select! {
                result = future => Ok(result),
                _ = cancellation_token_clone.cancelled() => Err(Cancelled)
            }
        });

        Self {
            handle: Arc::new(handle),
            cancellation_token,
        }
    }

    pub async fn cancel(&self) {
        self.cancellation_token.cancel();
    }
}
