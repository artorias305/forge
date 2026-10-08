# Forge

Forge is a dynamically typed interpreted programming language.

Run the REPL with no arguments, or execute one or more source files in order:

```bash
forge program.fg
forge definitions.fg main.fg
```

All files in one invocation share the same interpreter state. Use `forge --help`
for the complete CLI usage and `forge --version` to print the current version.

The REPL reads one line at a time, so write blocks on a single line there (or
run them from a file). It has two built-in commands: `.help` lists them, and
`.tokens` prints the token stream of the rest of the line.

## Statements

### Variables

`let` binds a name to a value. Names have no declared type and can be reused:

```forge
let count = 10
let greeting = "hello"
let ready = true
let count = 20
```

Reading an undefined variable is a runtime error.

### Print

`print(expr)` writes the value of `expr` followed by a newline:

```forge
print("hello, forge")
print(10 + 5)
```

### Conditionals

`if` takes a condition and a block, with an optional `else` or `else if`:

```forge
if 10 > 5 {
    print("big")
} else if 10 > 1 {
    print("small")
} else {
    print("tiny")
}
```

The condition must evaluate to a boolean; any other value is a runtime error.

## Expressions

Forge supports numeric, string, and boolean expressions.

Boolean literals are `true` and `false`. Boolean negation uses `!`:

```forge
print(!false)
```

Comparison operators are:

`== != < <= > >=`

Comparisons return a boolean value:

```forge
print(10 > 5)
print("forge" == "forge")
print(true != false)
```

Arithmetic operators are `+ - * /`, and `+` also concatenates strings:

```forge
print(7 - 3 * 2)
print("forge" + "!")
```

Arithmetic is integer-only. Floating-point literals are supported for
comparisons (`print(1 == 1.0)` prints `true`) but using them in arithmetic is a
runtime error. Integer division truncates toward zero, and division by zero or
integer overflow are runtime errors.

String literals support the escapes `\" \\ \n \r \t`:

```forge
print("line one\nline two")
```

### Operator precedence

From tightest to loosest binding, with left associativity:

1. unary `+ - !`
2. `* /`
3. `+ -`
4. `< <= > >=`
5. `== !=`

Parentheses group subexpressions: `print((1 + 2) * 3)` prints `9`.

### Type errors

Because Forge is dynamically typed, operand types are checked at runtime.
Operations on mismatched types report an error instead of coercing:

```forge
print("a" + 1)
```

```
cannot use '+' with string and number
```

The same rule covers the other combinations, for example `true == 1` reports
`cannot use '==' with boolean and number`, and `"a" < "b"` reports
`cannot use '<' with string and string`.

`+` concatenates only two strings, and ordering comparisons (`< <= > >=`)
apply only to numbers. Equality (`== !=`) works within each type: numbers,
strings, or booleans.
