use std::{io, time::Instant};

use crate::{
    START_POSITION_FEN,
    movegen::{Move, get_move_string},
    position::Position,
    search::search,
};

fn read_line() -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(n) => return input,
        Err(error) => panic!("error: {error}"),
    }
}

fn parse_move(move_string: &str, position: &mut Position) -> Move {
    // e2e4 e7e5 g1f3 b8c6 f1b5
    let from_file = move_string.chars().nth(0).unwrap() as usize;
    let from_rank = move_string.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;
    let to_file = move_string.chars().nth(2).unwrap() as usize;
    let to_rank = move_string.chars().nth(3).unwrap().to_digit(10).unwrap() as usize;

    let from_square = (from_file - 97) + ((8 - from_rank) * 16);
    println!("{from_square}");
    let to_square = (to_file - 97) + ((8 - to_rank) * 16);
    println!("{to_square}");

    let moves = position.generate_legal_moves();
    for move_ in &moves {
        if move_.from == from_square && move_.to == to_square {
            return move_.clone();
        }
    }
    // return illegal move for now if no move is matched
    Move {
        from: 0x88,
        to: 0x88,
        promoted_piece: None,
        is_capture: false,
        is_enpassant: false,
        is_double_pawn: false,
        is_castling: false,
    }
}

fn handle_position(input: &String, position: &mut Position) {
    // > position startpos
    // > position startpos moves e2e4 e7e5 g1f3 b8c6 f1b5
    // > position fen 8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1
    // > position fen 8/3P3k/n2K3p/2p3n1/1b4N1/2p1p1P1/8/3B4 w - - 0 1 moves g4f6 h7g7 f6h5 g7g6 d1c2
    if input.contains("fen") {
        let fen_part = input.strip_prefix("position fen ").unwrap();
        *position = Position::from_fen(&fen_part);
    } else if input.contains("startpos") {
        *position = Position::from_fen(START_POSITION_FEN);
    }
    if input.contains("moves") {
        *position = Position::from_fen(START_POSITION_FEN);
        let index = input.find("moves").unwrap();
        let moves_part = &input[index + 6..];
        for move_string in moves_part.split(" ") {
            // TODO: Needs to parse these into Moves that get made in the position
            println!("{move_string}");
            let move_ = parse_move(move_string, position);
            position.make_move(&move_);
        }
        println!("{moves_part}");
    }
}

fn handle_go(input: &String, position: &mut Position) {
    // go depth x movetime y
    // > go depth 25
    // wtime <x>
    // Tell the engine that White has x ms left on the clock.
    // btime <x>
    // Tell the engine that Black has x ms left on the clock.
    // winc <x>
    // Tell the engine that White's increment per move in ms if x > 0.
    // binc <x>

    // default depth
    let mut depth: u32 = 4;

    // Parse depth
    if input.contains("depth") {
        let index = input.find("depth").unwrap();
        depth = input[index + 6..].trim().parse::<u32>().unwrap();
        println!("{depth}");
    }

    // Need to consider movetime, wtime, btime, and increments
    // basic time management

    // search needs to support basic time management, stopping early etc.

    let mut nodecount = 0;
    let start = Instant::now();
    let best_move = search(position, depth, &mut nodecount);
    let duration = start.elapsed().as_secs_f32();
    let nodes_per_sec = (nodecount as f32 / duration) as u64;

    println!("info nodes {}", nodecount);
    println!("info nps {}", nodes_per_sec);
    println!("bestmove {}", get_move_string(&best_move));
}

// Minimum UCI Requirements
// For SPRT testing platforms such as cutechess-cli, fast-chess and OpenBench, the minimum UCI requirements are as follows.

// Commands to support:

// go wtime <> btime <> winc <> binc <>
// position startpos
// position fen <fen>
// quit
// stop
// uci
// ucinewgame
// isready

pub fn uci_loop() {
    let mut position = Position::from_fen(START_POSITION_FEN);

    loop {
        let input = read_line();

        if input.contains("position") {
            handle_position(&input, &mut position);
            position.print();
        } else if input.contains("quit") {
            break;
        } else if input.contains("ucinewgame") {
            position = Position::from_fen(START_POSITION_FEN);
            position.print();
        } else if input.contains("isready") {
            println!("readyok");
        } else if input.contains("go") {
            handle_go(&input, &mut position);
        } else if input.contains("stop") {
            println!("Handle stop")
            // stops calculating as soon as possible
        } else if input.contains("uci") {
            println!("id name rustchess");
            println!("id author Eetu Rantala");
            println!("uciok");
        }
    }
}
