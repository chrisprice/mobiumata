#![cfg_attr(not(test), no_std)]

pub use mobiumata_automaton as automaton;
pub mod display;
pub mod network;
pub mod state;