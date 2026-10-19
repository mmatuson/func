#[cfg(not(feature = "persistent-ds-env"))]
mod basic;

#[cfg(feature = "persistent-ds-env")]
mod persistent;

#[cfg(not(feature = "persistent-ds-env"))]
pub use basic::{EnvironmentStack, Locals, SharedMutableEnvironment};

#[cfg(feature = "persistent-ds-env")]
pub use persistent::{EnvironmentStack, Locals, SharedMutableEnvironment};
