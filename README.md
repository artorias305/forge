# Forge

Forge is a dynamically typed interpreted programming language.

Run the REPL with no arguments, or execute one or more source files in order:

```bash
forge program.fg
forge definitions.fg main.fg
```

All files in one invocation share the same interpreter state. Use `forge --help`
for the complete CLI usage and `forge --version` to print the current version.

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
