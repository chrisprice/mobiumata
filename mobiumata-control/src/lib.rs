#![no_std]

use embassy_futures::select::{select3, select4};
use embassy_rp::{
    gpio::{Input, Pin, Pull},
    Peripheral,
};
use mobiumata_common::{
    automaton::{Rule, Wrap},
    state::{State, Step},
};

pub struct Buttons<
    'a,
    P0: Pin,
    P1: Pin,
    P2: Pin,
    P3: Pin,
    P4: Pin,
    P5: Pin,
    P6: Pin,
    P7: Pin,
    P8: Pin,
    P9: Pin,
    P10: Pin,
> {
    rule_0: Input<'a, P0>,
    rule_1: Input<'a, P1>,
    rule_2: Input<'a, P2>,
    rule_3: Input<'a, P3>,
    rule_4: Input<'a, P4>,
    rule_5: Input<'a, P5>,
    rule_6: Input<'a, P6>,
    rule_7: Input<'a, P7>,
    wrap_0: Input<'a, P8>,
    wrap_1: Input<'a, P9>,
    step: Input<'a, P10>,
}

impl<
        'a,
        P0: Pin,
        P1: Pin,
        P2: Pin,
        P3: Pin,
        P4: Pin,
        P5: Pin,
        P6: Pin,
        P7: Pin,
        P8: Pin,
        P9: Pin,
        P10: Pin,
    > Buttons<'a, P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10>
{
    pub fn new(
        rule_0: impl Peripheral<P = P0> + 'a,
        rule_1: impl Peripheral<P = P1> + 'a,
        rule_2: impl Peripheral<P = P2> + 'a,
        rule_3: impl Peripheral<P = P3> + 'a,
        rule_4: impl Peripheral<P = P4> + 'a,
        rule_5: impl Peripheral<P = P5> + 'a,
        rule_6: impl Peripheral<P = P6> + 'a,
        rule_7: impl Peripheral<P = P7> + 'a,
        wrap_0: impl Peripheral<P = P8> + 'a,
        wrap_1: impl Peripheral<P = P9> + 'a,
        step: impl Peripheral<P = P10> + 'a,
    ) -> Self {
        let mut rule_0 = Input::new(rule_0, Pull::Up);
        rule_0.set_schmitt(true);
        let mut rule_1 = Input::new(rule_1, Pull::Up);
        rule_1.set_schmitt(true);
        let mut rule_2 = Input::new(rule_2, Pull::Up);
        rule_2.set_schmitt(true);
        let mut rule_3 = Input::new(rule_3, Pull::Up);
        rule_3.set_schmitt(true);
        let mut rule_4 = Input::new(rule_4, Pull::Up);
        rule_4.set_schmitt(true);
        let mut rule_5 = Input::new(rule_5, Pull::Up);
        rule_5.set_schmitt(true);
        let mut rule_6 = Input::new(rule_6, Pull::Up);
        rule_6.set_schmitt(true);
        let mut rule_7 = Input::new(rule_7, Pull::Up);
        rule_7.set_schmitt(true);
        let mut wrap_0 = Input::new(wrap_0, Pull::Up);
        wrap_0.set_schmitt(true);
        let mut wrap_1 = Input::new(wrap_1, Pull::Up);
        wrap_1.set_schmitt(true);
        let mut step = Input::new(step, Pull::Up);
        step.set_schmitt(true);
        Self {
            rule_0,
            rule_1,
            rule_2,
            rule_3,
            rule_4,
            rule_5,
            rule_6,
            rule_7,
            wrap_0,
            wrap_1,
            step,
        }
    }

    pub async fn wait_for_any_edge(&mut self) -> () {
        select3(
            select4(
                self.rule_0.wait_for_any_edge(),
                self.rule_1.wait_for_any_edge(),
                self.rule_2.wait_for_any_edge(),
                self.rule_3.wait_for_any_edge(),
            ),
            select4(
                self.rule_4.wait_for_any_edge(),
                self.rule_5.wait_for_any_edge(),
                self.rule_6.wait_for_any_edge(),
                self.rule_7.wait_for_any_edge(),
            ),
            select3(
                self.wrap_0.wait_for_any_edge(),
                self.wrap_1.wait_for_any_edge(),
                self.step.wait_for_any_edge(),
            ),
        )
        .await;
    }

    pub fn read_state(&mut self) -> State {
        let mut rule = 0;
        if self.rule_0.is_high() {
            rule |= 1 << 0;
        }
        if self.rule_1.is_high() {
            rule |= 1 << 1;
        }
        if self.rule_2.is_high() {
            rule |= 1 << 2;
        }
        if self.rule_3.is_high() {
            rule |= 1 << 3;
        }
        if self.rule_4.is_high() {
            rule |= 1 << 4;
        }
        if self.rule_5.is_high() {
            rule |= 1 << 5;
        }
        if self.rule_6.is_high() {
            rule |= 1 << 6;
        }
        if self.rule_7.is_high() {
            rule |= 1 << 7;
        }
        let rule = Rule::new(rule);

        let wrap = if self.wrap_0.is_high() {
            Wrap::Zero
        } else {
            if self.wrap_1.is_high() {
                Wrap::One
            } else {
                Wrap::Wrap
            }
        };

        let step = Step::new(self.step.is_high());

        State { rule, wrap, step }
    }
}
