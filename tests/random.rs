use quickcheck::{Arbitrary, Gen, quickcheck};

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
    fn arbitrary(g: &mut Gen) -> Self {
        g.choose(COMMAND_FREQS).copied().unwrap_or(Command::Push)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item=Self>> {
        use Command::*;

        let v = match self {
            PushPopMin | ReplaceMin => vec![Push, PopMin],
            PushPopMax | ReplaceMax => vec![Push, PopMax],
            _ => vec![],
        };

        Box::new(v.into_iter())
    }
}

const COMMAND_FREQS: &[Command] = {
    use Command::*;
    &[
        Push, Push, Push,
        PopMin,
        PopMax,
        PushPopMin,
        PushPopMax,
        ReplaceMin,
        ReplaceMax,
    ]
};

impl<T: Arbitrary> Arbitrary for Script<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        Script((0 .. SCRIPT_LENGTH)
            .map(|_| (Command::arbitrary(g), T::arbitrary(g)))
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
            Push       => {
                r.push(e1);
                f.push(e2);
                true
            }
            PopMin     => r.pop_min() == f.pop_min(),
            PopMax     => r.pop_max() == f.pop_max(),
            PushPopMin => r.push_pop_min(e1) == f.push_pop_min(e2),
            PushPopMax => r.push_pop_max(e1) == f.push_pop_max(e2),
            ReplaceMin => r.replace_min(e1) == f.replace_min(e2),
            ReplaceMax => r.replace_max(e1) == f.replace_max(e2),
        }
    }
}
