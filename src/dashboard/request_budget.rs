use super::storage::AppError;
use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::{Duration, Instant},
};

const ACTIVE: u8 = 0;
const CANCELLED: u8 = 1;
const FINISHED: u8 = 2;

#[derive(Clone)]
pub(crate) struct Budget {
    deadline: Instant,
    state: Arc<AtomicU8>,
}

impl Budget {
    pub(crate) fn new(deadline: Instant) -> Self {
        Self {
            deadline,
            state: Arc::new(AtomicU8::new(ACTIVE)),
        }
    }

    pub(crate) fn cancel(&self) -> bool {
        matches!(
            self.state
                .compare_exchange(ACTIVE, CANCELLED, Ordering::Relaxed, Ordering::Relaxed),
            Ok(_) | Err(CANCELLED)
        )
    }

    pub(crate) fn check(&self) -> Result<(), AppError> {
        if self.state.load(Ordering::Relaxed) == CANCELLED {
            Err(cancelled())
        } else if Instant::now() >= self.deadline {
            Err(timed_out())
        } else {
            Ok(())
        }
    }

    pub(crate) fn finish_read(&self) -> Result<(), AppError> {
        self.state
            .compare_exchange(ACTIVE, FINISHED, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_| cancelled())?;
        self.check()
    }
}

pub(crate) fn cancelled() -> AppError {
    AppError::new("REQUEST_CANCELLED", "The read request was cancelled.")
}

fn timed_out() -> AppError {
    AppError::new("REQUEST_TIMEOUT", "The runtime request deadline expired.")
}

thread_local! {
    static CURRENT: RefCell<Option<Budget>> = const { RefCell::new(None) };
}

pub(crate) fn scoped<T>(budget: &Budget, operation: impl FnOnce() -> T) -> T {
    struct Restore(Option<Budget>);
    impl Drop for Restore {
        fn drop(&mut self) {
            CURRENT.replace(self.0.take());
        }
    }
    let _restore = Restore(CURRENT.replace(Some(budget.clone())));
    operation()
}

pub(super) fn check() -> Result<(), AppError> {
    CURRENT.with_borrow(|budget| budget.as_ref().map_or(Ok(()), Budget::check))
}

pub(super) fn remaining(cap: Duration) -> Result<Duration, AppError> {
    CURRENT.with_borrow(|budget| match budget {
        Some(budget) => {
            budget.check()?;
            let remaining = budget.deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                Err(timed_out())
            } else {
                Ok(remaining.min(cap))
            }
        }
        None => Ok(cap),
    })
}
