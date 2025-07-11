# Grippy

Grip theoretic twisty puzzle algorithm analysis tool

## Usage

1. `git clone https://github.com/HactarCE/grippy && cd grippy`
2. `cargo run --release`
3. Enter an algorithm
4. Enter relations

## Example algorithms

```
[R, U] [U2, R]          // sune
[R, U'] [F, R'] [U, F'] // identity on 3x3x3
[R, U]                  // sexy
[R, U] [R', F]          // sexy + sledge
[[R', U'], [L, U]]      // megaminx U perm
[[U': R'], L]           // niklas
[[R L: U2], U]          // U2 comm (RKT parity for 3^4)
```

Grouping, commutators, and conjugates are all allowed.

## Example relations

```
U = F * R
R = U * F
F = R * U
L = F * U
F = U * L

F = F * [R: U']
U = F * [R: U'] F
R = F * U' R' F
```

- At the start of a line is is a grip
- After `=` is a grip
- After `*` is a move sequence, which may be multiple moves and may use grouping/commutators/conjugates
- `*` and `×` are equivalent; both are accepted.
