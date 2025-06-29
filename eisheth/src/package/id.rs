use std::sync::atomic::{AtomicU64, Ordering};

pub static PACKAGE_ID_GENERATOR: PackageIdGenerator = PackageIdGenerator::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackageId(u64);

// TODO we have a couple of generators like this, make it an abstraction?
pub struct PackageIdGenerator(AtomicU64);

impl PackageIdGenerator {
    const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub(crate) fn next(&self) -> PackageId {
        PackageId(self.0.fetch_add(1, Ordering::Relaxed))
    }
}
