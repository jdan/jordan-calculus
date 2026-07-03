# JordanCalculus

A language eerily similar to the Lambda Calculus that attempts to answer the question: What if a programming language were miserable to write?

As agentic coding appears in more of our daily workflows there is a commonly-held belief that languages/frameworks "in the training data" have a substantial advantage. I do not believe this to be true. As such, this is an experiment in programming with something decently foreign to most frontier models. (Caveat: They are trained on the entire corpus of Computer Science education so they know what the lambda calculus is and how to compile to it. We'll see.)

## Notation

A JordanCalculus program is an expression:

```text

variable ::= katakana katakana*
katakana ::= ア | イ | ウ | エ | ...
expression ::= variable
             | J variable ッ expression
             | expression 足す expression
             | 「 expression 」
```

## Usage

```sh
cargo run -- --expr 'Jアッア' out.wat
cargo run -- path/to/program.jc out.wat
```

The emitted module exports:

- `memory`
- `main() -> i32`, returning a runtime handle to the resulting closure/value

This is an MVP compiler/runtime for untyped call-by-value lambda calculus. Free variables trap at runtime.
