use labelr::retry::retry_octocrab;
use octocrab::Error as OctoError;
use snafu::Backtrace;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn retry_succeeds_after_transient_errors() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    let op = || {
        let c = c.clone();
        async move {
            let attempt = c.fetch_add(1, Ordering::SeqCst);
            if attempt < 2 {
                Err(OctoError::Other {
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "transient")),
                    backtrace: std::backtrace::Backtrace::capture().into(),
                })
            } else {
                Ok("ok".to_string())
            }
        }
    };

    let res = retry_octocrab(op, 5).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "ok".to_string());
}
