extern crate min_max_heap;
extern crate rand;
#[macro_use]
extern crate quickcheck;

use quickcheck::{Arbitrary, Gen};
use rand::prelude::*;
use rand::distributions::WeightedIndex;

mod fake_heap;

const SCRIPT_LENGTH: usize = 1000;

quickcheck! {
    fn prop_usize(script: Script<usize>) -> bool {
        script.check()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Command {
    Push,
    PopMin,
    PopMax,
    PushPopMin,
    PushPopMax,
    ReplaceMin,
    ReplaceMax,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Script<T>(Vec<(Command, T)>);

impl Arbitrary for Command {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        use Command::*;

        let choices = [
            (3, Push),
            (1, PopMin),
            (1, PopMax),
            (1, PushPopMin),
            (1, PushPopMax),
            (1, ReplaceMin),
            (1, ReplaceMax),
        ];

        let dist = WeightedIndex::new(choices.iter().map(|p| p.0)).unwrap();

        choices[dist.sample(g)].1
    }
}

impl<T: Arbitrary> Arbitrary for Script<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Script((0 .. SCRIPT_LENGTH)
            .map(|_| (Command::arbitrary(g), T::arbitrary(g)))
            .collect())
    }

    fn shrink(&self) -> Box<Iterator<Item=Self>> {
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
        script.0.iter().all(|&(cmd, ref elt)|
            self.check_command(cmd, elt) && self.check_extrema())
    }

    fn check_extrema(&self) -> bool {
        self.real.peek_min() == self.fake.peek_min() &&
            self.real.peek_max() == self.fake.peek_max()
    }

    fn check_command(&mut self, cmd: Command, elt: &T) -> bool {
        use Command::*;

        let e1 = elt.clone();
        let e2 = elt.clone();
        let r  = &mut self.real;
        let f  = &mut self.fake;

        match cmd {
            Push       => r.push(e1) == f.push(e2),
            PopMin     => r.pop_min() == f.pop_min(),
            PopMax     => r.pop_max() == f.pop_max(),
            PushPopMin => r.push_pop_min(e1) == f.push_pop_min(e2),
            PushPopMax => r.push_pop_max(e1) == f.push_pop_max(e2),
            ReplaceMin => r.replace_min(e1) == f.replace_min(e2),
            ReplaceMax => r.replace_max(e1) == f.replace_max(e2),
        }
    }
}
