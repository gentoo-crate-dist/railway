use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use gdk::glib;

/// Limit a request based on a timeout.
#[derive(Debug, Clone)]
pub struct RequestLimiter<T> {
    value: Arc<Mutex<Option<T>>>,
    locked_until: Arc<Mutex<Instant>>,
    timeout: Duration,
    pending_request: Arc<AtomicBool>,
}

impl<T> RequestLimiter<T> {
    /// Create the limiter with the given timeout.
    pub fn new(timeout: Duration) -> Self {
        log::trace!(
            "Initializing request limiter with {} milliseconds.",
            timeout.as_millis()
        );
        Self {
            value: Arc::new(Mutex::new(None)),
            locked_until: Arc::new(Mutex::new(Instant::now())),
            timeout,
            pending_request: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create a request for the data.
    ///
    /// If a request is allowed to go through, the data that should be requested will be returned. Note that this may not be the data given, e.g. when there were other requests in the meantime which overwrote the value.
    /// If there is no pending request, this will directly allow the request and block other requests for the timeout.
    /// If the store is already locked, but no request is pending, the thread will sleep until it is allowed to request and then return.
    /// If the store is already locked and there is a pending request, it will instantly return `None`.
    pub async fn request(&self, value: T) -> Option<T> {
        let to_sleep = {
            let mut stored = self.value.lock().expect("poisoned value lock");
            let direct = stored.is_none();
            *stored = Some(value);

            let mut locked = self.locked_until.lock().expect("poisoned time lock");
            let now = Instant::now();

            if direct && *locked < now {
                // No previous request. Can directly pass.
                *locked = now + self.timeout;
                log::trace!("Previous request already finished and no data left. Locking for {} milliseconds.", self.timeout.as_millis());
                Some(Duration::ZERO)
            } else if self.pending_request.load(Ordering::SeqCst) {
                // Already pending. Return.
                log::trace!("There is already a request pending. Cancelling request.");
                None
            } else {
                // Still locked, but no pending requests. Timeout.
                log::trace!("It is still locked, but no pending request. Becoming pending thread. Sleeping for {} milliseconds.", (*locked - now).as_millis());
                Some(*locked - now)
            }
            // Drop all the locks.
        }?;

        if !to_sleep.is_zero() {
            self.pending_request.store(true, Ordering::SeqCst);
            glib::timeout_future(to_sleep).await;

            let mut locked = self.locked_until.lock().expect("poisoned time lock");
            let now = Instant::now();
            *locked = now + self.timeout;

            self.pending_request.store(false, Ordering::SeqCst);
        }

        // Return the stored value, leaving none.
        self.value.lock().expect("poisoned value lock").take()
    }
}
