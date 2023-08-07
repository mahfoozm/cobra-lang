<h1 align="center">Cobra</h1>


<p align="center">
  <img src="https://github.com/mahfoozm/cobra-lang/assets/95328615/d716d1e2-4a06-4b2a-8b2a-4d9d8472b6fa" />
</p>

My attempt at implementing a simple language using LLVM components in Rust.

### Features:
- Python-like syntax
- Protoype declarations
- Functions
- Conditionals
- Loops
- Comments
- Error handling (soon)

## Usage

```bash
# Run cobra program from file.
cargo run <filename>

# Run code interactively (parsing from stdin)
cargo run
```

## Example

```python
# Prints the fibonacci sequence up to n.
def fib(n)
  if n < 3 then
    1
  else
    fib(n-1) + fib(n-2);

fib(10);
```

```bash
$ cargo run fib.ks
Parse fib.ks.
[src/parser.rs:185] self.cur_tok() = Then
Parse 'def'
define double @fib(double %n) {
block:
  %fcmpult = fcmp ult double %n, 3.000000e+00
  br i1 %fcmpult, label %block5, label %block2

block2:                                           ; preds = %block
  %fsub = fadd double %n, -1.000000e+00
  %call = call double @fib(double %fsub)
  %fsub3 = fadd double %n, -2.000000e+00
  %call4 = call double @fib(double %fsub3)
  %fadd = fadd double %call, %call4
  br label %block5

block5:                                           ; preds = %block, %block2
  %phi = phi double [ %fadd, %block2 ], [ 1.000000e+00, %block ]
  ret double %phi
}
define double @__anon_expr() {
block:
  %call = call double @fib(double 1.000000e+01)
  ret double %call
}
Evaluated to 55
```
