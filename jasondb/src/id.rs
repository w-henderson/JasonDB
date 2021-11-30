use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEEDED: AtomicBool = AtomicBool::new(false);

pub fn generate_id() -> String {
    if !SEEDED.load(Ordering::SeqCst) {
        unsafe { srand() };
        SEEDED.store(true, Ordering::SeqCst);
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let rand = unsafe { rand() };

    format!("{:X}{:X}", timestamp, rand)
}

extern "C" {
    fn srand() -> u32;
    fn rand() -> u32;
}
