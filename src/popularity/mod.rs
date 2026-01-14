/// He is responsible for ensuring that nodes exchange information about "trends".
///
/// Thanks to him, the node learns that a file has become a hit on the other side of the
///  planet, and can prepare in advance (replicate it).
pub mod exchanger;
pub mod metrics;
pub mod ranking;
