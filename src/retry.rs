use octocrab::Error as OctoError;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

pub async fn retry_octocrab<T, F, Fut>(mut op: F, attempts: usize) -> Result<T, OctoError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, OctoError>>,
{
    for i in 0..attempts {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                // Decide whether to retry
                let should_retry = match &e {
                    OctoError::GitHub { source, .. } => {
                        let code = source.status_code.as_u16();
                        code == 429 || code >= 500
                    }
                    OctoError::Hyper { .. }
                    | OctoError::Http { .. }
                    | OctoError::Service { .. }
                    | OctoError::Other { .. } => true,
                    _ => false,
                };

                if !should_retry || i + 1 == attempts {
                    return Err(e);
                }

                // Exponential backoff with jitter to avoid thundering herd.
                let base_secs = 2u64.pow(i as u32);
                let mut rng = rand::thread_rng();
                let jitter_ms = rng.gen_range(0..(base_secs * 1000));
                let backoff = Duration::from_millis(base_secs * 1000 + jitter_ms);
                sleep(backoff).await;
            }
        }
    }
    unreachable!()
}
