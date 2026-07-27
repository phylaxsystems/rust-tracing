use rust_tracing::trace;
use std::sync::{
    Arc,
    atomic::AtomicBool,
};

fn main() {
    let term = Arc::<AtomicBool>::default();

    let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term));

    let _guard = trace();
}
