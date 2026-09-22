# Associated types vs generic params — build both

Implement a Parser trait twice. Version A: trait Parser<Input, Output> { fn parse(&self, input: Input) -> Option<Output>; }. Version B: trait Parser { type Input; type Output; fn parse(&self, input: Self::Input) -> Option<Self::Output>; }. Build a CsvParser and a JsonLineParser implementing both versions. Then write a fn run_parser that accepts a parser — see why Version B is much easier to use as a generic bound: Parser vs Parser<String, Vec<String>>. Write a comment explaining when each is appropriate.

> What this cements: Associated types = one impl per type (Iterator, Add). Generic params = multiple impls per type (From<T> can be implemented for many T). This distinction confuses almost everyone the first time.
