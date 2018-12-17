extern crate min_max_heap;
extern crate rand;
#[macro_use]
extern crate quickcheck;

use quickcheck::{Arbitrary, Gen};
use rand::Rng;

mod fake_heap;

const SCRIPT_LENGTH: usize = 1000;

quickcheck! {
    fn prop_usize(script: Script<usize>) -> bool {
        script.check()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Command<T> {
    Push(T),
    PopMin,
    PopMax,
    PushPopMin(T),
    PushPopMax(T),
    ReplaceMin(T),
    ReplaceMax(T),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Script<T>(Vec<Command<T>>);

impl<T: Arbitrary> Arbitrary for Command<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        use Command::*;

        let choice = g.gen_range(1, 91);
        let mut element = || T::arbitrary(g);

        match choice {
            01...30 => Push(element()),
            31...40 => PopMin,
            41...50 => PopMax,
            51...60 => PushPopMin(element()),
            61...70 => PushPopMax(element()),
            71...80 => ReplaceMin(element()),
            81...90 => ReplaceMax(element()),
            _       => unreachable!(),
        }
    }
}

impl<T: Arbitrary> Arbitrary for Script<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Script((0 .. SCRIPT_LENGTH)
            .map(|_| Command::<T>::arbitrary(g))
            .collect())
    }

    fn shrink(&self) -> Box<dyn Iterator<Item=Self>> {
        Box::new(self.0.shrink().map(Script))
    }
}

impl<T: Clone + Ord + ::std::fmt::Debug> Script<T> {
    fn check(&self) -> bool {
        let mut tester = Tester::new();
        tester.check_script(self)
    }
}

struct Tester<T> {
    real: min_max_heap::MinMaxHeap<T>,
    fake: fake_heap::FakeHeap<T>,
}

impl<T: Clone + Ord> Tester<T> {
    fn new() -> Self {
        Tester {
            real: min_max_heap::MinMaxHeap::new(),
            fake: fake_heap::FakeHeap::new(),
        }
    }

    fn check_script(&mut self, script: &Script<T>) -> bool {
        script.0.iter().all(|cmd|
            self.check_command(cmd) && self.check_extrema())
    }

    fn check_extrema(&self) -> bool {
        self.real.peek_min() == self.fake.peek_min() &&
            self.real.peek_max() == self.fake.peek_max()
    }

    fn check_command(&mut self, cmd: &Command<T>) -> bool {
        use Command::*;

        match cmd {
            Push(element) =>
                self.real.push(element.clone()) == self.fake.push(element.clone()),

            PopMin =>
                self.real.pop_min() == self.fake.pop_min(),

            PopMax =>
                self.real.pop_max() == self.fake.pop_max(),

            PushPopMin(e) =>
                self.real.push_pop_min(e.clone()) == self.fake.push_pop_min(e.clone()),

            PushPopMax(e) =>
                self.real.push_pop_max(e.clone()) == self.fake.push_pop_max(e.clone()),

            ReplaceMin(e) =>
                self.real.replace_min(e.clone()) == self.fake.replace_min(e.clone()),

            ReplaceMax(e) =>
                self.real.replace_max(e.clone()) == self.fake.replace_max(e.clone()),
        }
    }
}
