use defmt::Format;
use serde::{Deserialize, Serialize};

use crate::automaton::{Rule, Wrap};

#[derive(Clone, Copy, Debug, PartialEq, Format, Serialize, Deserialize)]
pub struct Step(bool);

impl Step {
    pub fn new(step: bool) -> Self {
        Self(step)
    }

    pub fn inner(&self) -> bool {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Format, Serialize, Deserialize)]
pub struct State {
pub rule: Rule,
    pub wrap: Wrap,
    pub step: Step,
}

impl Default for State {
    fn default() -> Self {
        Self {
            rule: Rule::new(30),
            wrap: Wrap::Wrap,
            step: Step::new(false),
        }
    }
}