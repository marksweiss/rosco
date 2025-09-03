#[allow(clippy::module_inception)]
pub (crate) mod envelope;
pub mod envelope_pair;

pub(crate) use envelope::{Envelope, EnvelopeBuilder};