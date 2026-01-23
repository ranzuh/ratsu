<div align="center">
  <img width="200" alt="ratsu-logo" src="https://github.com/user-attachments/assets/750647f6-0078-44a3-a2e5-297c1abf004f" style="display:     block; margin: 0 auto" />
  <h3>Ratsu</h3>
  <p>Chess engine written in Rust</p>
</div>

## Features
- 0x88 board representation
- Iterative deepening negamax search
- Quiescence search
- Evaluation using material, piece-square tables, and pawn structures
- Transposition table with Zobrist hashing
- Move ordering using PV move, TT move, MVV-LVA, Killer moves, and History heuristic
- Check extension
- Null move pruning
- Principal variation search
- Detect repetitions and fifty-move rule
- Basic UCI support

## Build & Develop

Quick build & run

```shell
cargo run
```

Build for release

```shell
cargo build -r
```

Run tests

```shell
cargo test
```

Format code

```shell
cargo fmt
```
