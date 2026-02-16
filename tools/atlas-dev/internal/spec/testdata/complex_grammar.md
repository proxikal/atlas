# Complex Grammar Test

## Grammar Rules

```ebnf
program = statement+
statement = declaration | expression ";"
declaration = "let" identifier "=" expression ";"
expression = literal | binary_op | unary_op
binary_op = expression operator expression
unary_op = operator expression
operator = "+" | "-" | "*" | "/"
literal = number | string | boolean
identifier = letter { letter | digit }
number = digit+
letter = "a" | "b" | "c"
digit = "0" | "1" | "2"
```

## Invalid Grammar

```grammar
bad_rule = unclosed_bracket [
another_bad = mismatched (]
circular_a = circular_b
circular_b = circular_a
```
