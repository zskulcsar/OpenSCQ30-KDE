use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct Operation {
    pub generation: u64,
    pub cancellation: CancellationToken,
}

#[derive(Default)]
pub struct OperationLifecycle {
    generation: AtomicU64,
    active: Mutex<CancellationToken>,
}

impl OperationLifecycle {
    pub fn begin(&self) -> Operation {
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let mut active = self.active.lock().expect("operation lock poisoned");
        active.cancel();
        let cancellation = CancellationToken::new();
        *active = cancellation.clone();
        Operation {
            generation,
            cancellation,
        }
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.generation.load(Ordering::Acquire) == generation
    }

    pub fn cancel(&self) {
        self.begin();
    }
}
