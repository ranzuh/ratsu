<div align="center">
  <img width="200" alt="ratsu-logo" src="https://github.com/user-attachments/assets/750647f6-0078-44a3-a2e5-297c1abf004f" style="display:     block; margin: 0 auto" />
  <h3>Ratsu</h3>
  <p>Chess engine written in Rust</p>
</div>

## Features

* 0x88 board representation and move generation
* Iterative deepening negamax search
* Quiescence search
* Evaluation using Efficiently updated neural network (NNUE)
* Transposition table with Zobrist hashing
* Move ordering using TT move, MVV-LVA, Killer moves, and History heuristic
* Check extension
* Null move pruning
* Principal variation search
* Late move reductions
* Aspiration windows
* Futility pruning
* Detect repetitions and fifty-move rule
* Basic UCI support

## Build & Develop

Quick build & run

```
cargo run
```

Build for release

```
cargo build -r
```

Run tests

```
cargo test
```

Format code

```
cargo fmt
```

## Credit

Thanks to

* Maksim Korzh for his Bitboard CHESS ENGINE in C and 0x88 MOVE GENERATOR youtube series which inspired me to start programming chess engines.
* Chess Programming Wiki for all the knowledge
* Authors of TSCP, Blunder, Rustic
* The folks at chess programming discord servers
* The CCRL team and other chess engine testers for testing and rating Ratsu

