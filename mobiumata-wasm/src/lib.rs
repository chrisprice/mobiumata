#![no_std]

use core::array;

use mobiumata_automaton::{ElementaryCellularAutomaton, Rule, Wrap};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn ecm(rule: u8, wrap: Wrap, state: u8) -> (ElementaryCellularAutomaton, [bool; 8]) {
    let rule = Rule::new(rule);
    let automaton = ElementaryCellularAutomaton::new(wrap, rule);
    let state: [bool; 8] = array::from_fn(|i| (state >> i) & 1 == 1);
    (automaton, state)
}

fn ecm_next(rule: u8, state: u8, wrap: Wrap) -> u8 {
    let (automaton, state) = ecm(rule, wrap, state);
    let mut next_state = [false; 8];
    automaton.next(&state, &mut next_state);
    next_state.iter().enumerate().fold(0, |acc, (i, &cell)| acc | (cell as u8) << i)
}

#[no_mangle]
pub extern fn ecm_next_zero(rule: u8, state: u8) -> u8 {
    ecm_next(rule, state, Wrap::Zero)
}

#[no_mangle]
pub extern fn ecm_next_one(rule: u8, state: u8) -> u8 {
    ecm_next(rule, state, Wrap::One)
}

#[no_mangle]
pub extern fn ecm_next_wrap(rule: u8, state: u8) -> u8 {
    ecm_next(rule, state, Wrap::Wrap)
}

fn ecm_period(rule: u8, state: u8, wrap: Wrap) -> u8 {
    let (automaton, state) = ecm(rule, wrap, state);
    automaton.period::<8, 192>(&state).unwrap_or(0) as u8
}

#[no_mangle]
pub extern "C" fn ecm_one_period(rule: u8, state: u8) -> u8 {
    ecm_period(rule, state, Wrap::One)
}

#[no_mangle]
pub extern "C" fn ecm_wrap_period(rule: u8, state: u8) -> u8 {
    ecm_period(rule, state, Wrap::Wrap)
}

// Refactor existing ecm_zero_period to use ecm_period
#[no_mangle]
pub extern "C" fn ecm_zero_period(rule: u8, state: u8) -> u8 {
    ecm_period(rule, state, Wrap::Zero)
}