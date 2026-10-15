//! Abax core: the soroban model, the bead-move engine that knows every
//! complement rule, the multiplication and division procedures, the
//! curriculum, the drill generators and the adaptive scheduler.
//! No UI dependencies.

pub mod arith;
pub mod curriculum;
pub mod generator;
pub mod moves;
pub mod scheduler;
pub mod soroban;
pub mod stats;
pub mod store;

pub use moves::{Finger, Rule, Step, StepKind};
pub use soroban::{Rod, Soroban};
