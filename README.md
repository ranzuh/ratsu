<div align="center">
  <img width="200" alt="ratsu-logo" src="https://github.com/user-attachments/assets/750647f6-0078-44a3-a2e5-297c1abf004f" style="display:     block; margin: 0 auto" />
  <h3>Ratsu</h3>
  <p>Chess engine written in Rust</p>
</div>

## Ratings

| Version     | Estimated Rating (Elo) | CCRL Blitz Rating (Elo) | CCRL 40/15 Rating (ELo)          |
| ----------- | -----------------------|-------------------------|----------------------------------|
| 0.9.0       | 1800-1900              | 1742                    | -                                |
| 1.0.0       | -                      | -                       | -                                |
| 1.1.0       | 2023                   | 2021                    | -                                |
| 1.2.0       | 2250                   | -                       | -                                |
| 2.0.0*      | 2643                   | 2658                    | 2726                             |
| 2.1.0       | 2833                   | -                       | -                                |

* [CCRL Blitz Ratings list](https://computerchess.org.uk/404/)
* [CCRL 40/15 Ratings list](https://computerchess.org.uk/4040/)

\* NNUE evaluation was introduced in version 2.0.0 - previous versions used hand crafted evaluation.

## Features

* 0x88 board representation and move generation
* Iterative deepening negamax search
* Quiescence search
* Evaluation using an NNUE (Efficiently Updatable Neural Network)
* Transposition table with Zobrist hashing
* Move ordering using TT move, MVV-LVA, Killer moves, and History heuristic
* Check extension
* Null move pruning
* Principal variation search
* Late move reductions
* Aspiration windows
* Futility pruning
* Reverse futility pruning (Static null move pruning)
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

