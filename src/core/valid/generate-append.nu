#! /usr/bin/env nu

# Generates ./append.rs file for specified N number of possible variants
# If needed change the N and run the script with nushell

const N = 20

mut result = "
pub trait Append<A> {
    type Out;
    fn append(self, a: A) -> Self::Out;
}

impl<A0, A1> Append<A1> for (A0,) {
    type Out = (A0, A1);
    fn append(self, a1: A1) -> Self::Out {
        let (a0,) = self;
        (a0, a1)
    }
}
"

for i in 2..$N {
    let prev = 0..($i - 1) | each {|i| $"A($i)"} | str join ", "
    let current = $"A($i)"
    let next = $prev + ", " + $current

    $result += $"
impl<($next)> Append<($current)> for \(($prev)\) {
    type Out = \(($next)\);
    fn append\(self, ($current | str downcase): ($current)\) -> Self::Out {
        let \(($prev | str downcase)\) = self;
        \(($next | str downcase)\)
    }
}
    "
}

$result | save -f append.rs
