# JordanCalculus

A language eerily similar to the Lambda Calculus that attempts to answer the question: What if a programming language were miserable to write?

As agentic coding appears in more of our daily workflows there is a commonly-held belief that languages/frameworks "in the training data" have a substantial advantage. I do not believe this to be true. As such, this is an experiment in programming with something decently foreign to most frontier models. (Caveat: They are trained on the entire corpus of Computer Science education so they know what the lambda calculus is and how to compile to it. We'll see.)

## Notation

A JordanCalculus program is an expression:

```text
program ::= line*
line ::= expression | definition | import | comment | blank-line
comment ::= え any-text
definition ::= 上げる variable は expression
import ::= 貰う variable は path

variable ::= katakana katakana*
katakana ::= ア | イ | ウ | エ | ...
expression ::= variable
             | J variable ッ expression
             | expression 足す expression
             | 「 expression 」
```

Comments use `え` and must occupy an entire line, optionally preceded by whitespace:

```text
え this whole line is ignored
Jアッア
```

Newlines are otherwise treated like whitespace.

Top-level definitions use `上げる` and `は`:

```text
上げる アイデンティティ は Jアッア
アイデンティティ
```

Definitions do not add new core syntax. They are expanded before parsing. The example above becomes:

```text
「Jアイデンティティッアイデンティティ」足す「Jアッア」
```

Top-level imports use `貰う` and `は` with an unquoted file path relative to the current source file:

```text
貰う タス は prelude.jc
```

The imported file must contain a matching top-level definition:

```text
上げる タス は JムッJンッJフッJエッ...
```

Imports are also expanded before parsing; they do not add core expression syntax.

## Usage

```sh
cargo run -- --expr 'Jアッア' out.wat
cargo run -- path/to/program.jc out.wat
```

The emitted module exports:

- `memory`
- `main() -> i32`, returning a runtime handle to the resulting closure/value
- `main_as_i32() -> i32`, an adapter that treats the result as a Church numeral and returns a WebAssembly `i32`

`main_as_i32` does not add any syntax to JordanCalculus. It is only a generated WebAssembly adapter: it applies the program result to an internal increment function and zero.

This is an MVP compiler/runtime for untyped call-by-value lambda calculus. Free variables trap at runtime.
