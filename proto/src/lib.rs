mod items {
    include!(concat!(env!("OUT_DIR"), "/squeue.items.rs"));
}
pub use items::*;