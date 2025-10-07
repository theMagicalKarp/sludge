# Sludge 🛢️

_An experimental interpreted language oozing with simplicity and correctness._

<img align="right" src="./docs/assets/sludge.png" height="100px" alt="some sludge">

Sludge is an interpreted programming language focused on clarity, correctness,
and no surprises. Think of it as your friendly neighborhood language experiment,
it won’t do your taxes, but it might just help you rethink how languages should
feel.

## Philosophy

Sludge was built as an exploration into what it means to write code that’s
simple, predictable, and mathematically grounded. It aims to combine the
elegance of functional design with the raw, unfiltered goop of pragmatic
implementation.

## Features

> [!IMPORTANT]
> This project is mostly an experiment in designing and implementing my own
> interpreted programming language.

- 🧠 Functional – Functions are first-class, immutable values.
- 🔣 Primitive-only – Everything is raw and direct. No implicit magic.
- 🔧 No Operator Overloading – + means what it means, forever and always.
- 📦 Built-in Data Structures – List, HashMap, and HashSet are baked right in.
- 💬 REPL Included – Experiment interactively without ceremony.

## Example

```python
# A simple example of Sludge syntax (subject to change!)
let add = fn(amount) {
    return fn(value) {
      return amount + value
    }
}

let add_ten = add(10)

let nums = list(1, 2, 3, 4)
print(nums.map(add_ten)) # list(11, 12, 13, 14)
```

## Commands

```
sludge run path/to/file.sludge     # Run a Sludge program
sludge ast path/to/file.sludge     # Print the abstract syntax tree
sludge repl                        # Start the interactive REPL
```

## Development

Requirements

- [rust](https://rust-lang.org/) — for the compiler and runtime core
- [pest](https://pest.rs/) — for parsing and grammar definition
- [Make](https://www.gnu.org/software/make/manual/make.html) — for build
  automation

```
make build       # Build the debug version
make release     # Build the release version
make test        # Run all tests
make lint        # Check code formatting and clippy warnings
make fix         # Automatically fix lint issues

make run path/to/file.sludge  # Run a file
make ast path/to/file.sludge  # Print the AST
make repl                     # Launch the REPL
```

## Roadmap / TODO

- Module System — Capability to import and link multiple .sludge files
- Static Typing — Move towards optional or enforced type annotations
- Better Error Handling — Using monads for composable and expressive error flow
  (Monad (Wikipedia)
- Extended Types — Add float32, float64, u8, u32, u64, etc.
- System Interaction — Expose process I/O and environment variables
- Immutable by Default — Variables should be immutable unless explicitly
  declared mutable
