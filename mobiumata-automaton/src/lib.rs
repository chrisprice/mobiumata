#![no_std]

use core::usize;

use defmt::Format;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Format, Serialize, Deserialize)]
pub struct Rule(u8);

impl Rule {
    pub fn new(rule: u8) -> Self {
        Self(rule)
    }

    pub fn inner(&self) -> u8 {
        self.0
    }
}

pub struct ElementaryCellularAutomaton {
    pub wrap: Wrap,
    pub rule: Rule,
}

impl ElementaryCellularAutomaton {
    pub fn new(wrapping: Wrap, rule: Rule) -> Self {
        Self {
            wrap: wrapping,
            rule,
        }
    }

    pub fn next(&self, state: &[bool], next_state: &mut [bool]) {
        assert_eq!(state.len(), next_state.len());

        let len = state.len();

        for i in 0..len {
            let left = self.wrap.left(state, i);
            let center = state[i];
            let right = self.wrap.right(state, i);

            let index = (left as u8) << 2 | (center as u8) << 1 | right as u8;
            next_state[i] = (self.rule.0 >> index) & 1 == 1;
        }
    }

    pub fn period<const W: usize, const MAX: usize>(&self, initial_state: &[bool]) -> Option<usize> {
        assert_eq!(initial_state.len(), W);

        let mut state = [[false; W]; MAX];
        state[0].copy_from_slice(initial_state);

        for i in 1..MAX {
            self.next_row(&mut state, i);
        }

        let last_index = MAX - 1;
        for i in 1..MAX {
            if state[last_index] == state[last_index - i] {
                return Some(i);
            }
        }

        None
    }

    pub fn next_row<const W: usize, const H: usize>(
        &self,
        state: &mut [[bool; W]; H],
        index: usize,
    ) {
        assert!(H > 2);
        let (previous_row, next_row) = if index == 0 {
            let (tail, head) = state.split_at_mut(1);
            (head.last().unwrap(), tail.first_mut().unwrap())
        } else {
            let (head, tail) = state.split_at_mut(index);
            (head.last().unwrap(), tail.first_mut().unwrap())
        };
        self.next(previous_row, next_row);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Format, Serialize, Deserialize)]
pub enum Wrap {
    Wrap,
    Zero,
    One,
}

impl Wrap {
    fn left(&self, state: &[bool], i: usize) -> bool {
        if i > 0 {
            return state[i - 1];
        }
        match self {
            Wrap::Wrap => state[state.len() - 1],
            Wrap::Zero => false,
            Wrap::One => true,
        }
    }
    fn right(&self, state: &[bool], i: usize) -> bool {
        if i < state.len() - 1 {
            return state[i + 1];
        }
        match self {
            Wrap::Wrap => state[0],
            Wrap::Zero => false,
            Wrap::One => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elementary_cellular_automaton() {
        // https://mathworld.wolfram.com/Rule30.html
        let automaton = ElementaryCellularAutomaton::new(Wrap::Zero, Rule::new(30));
        let mut next_state = [false; 9];

        let mut state = [false, false, false, false, true, false, false, false, false];
        automaton.next(&state, &mut next_state);

        assert_eq!(
            next_state,
            [false, false, false, true, true, true, false, false, false]
        );

        state.copy_from_slice(&next_state);
        automaton.next(&state, &mut next_state);

        assert_eq!(
            next_state,
            [false, false, true, true, false, false, true, false, false]
        );
    }

    #[test]
    fn test_elementary_cellular_automaton_period() {
        let automaton = ElementaryCellularAutomaton::new(Wrap::Wrap, Rule::new(2));
        let initial_state = [false, false, false, false, false, false, false, true];

        assert_eq!(automaton.period::<8, 192>(&initial_state), Some(8));
    }

    #[test]
    fn test_elementary_cellular_automaton_next_row() {
        let automaton = ElementaryCellularAutomaton::new(Wrap::Wrap, Rule::new(2));
        let mut state = [[false; 8]; 192];
        state[0] = [false, false, false, false, false, false, false, true];

        automaton.next_row(&mut state, 1);
        assert_eq!(
            state[0],
            [false, false,  false, false, false, false, false, true]
        );

        automaton.next_row(&mut state, 2);
        assert_eq!(
            state[1],
            [false, false, false, false, false, false, true, false]
        );

        automaton.next_row(&mut state, 3);
        assert_eq!(
            state[2],
            [false, false, false, false, false, true, false, false]
        );
    }
}
