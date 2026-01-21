use std::io;

use regex::Regex;

use crate::{
    START_POSITION_FEN,
    hash::TranspositionTable,
    movegen::{Move, get_move_string},
    perft::run_perft,
    piece::{BISHOP, BLACK, KNIGHT, QUEEN, ROOK, WHITE},
    position::Position,
    search::Search,
};

fn read_line() -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input,
        Err(error) => panic!("error: {error}"),
    }
}

fn parse_move(move_string: &str, position: &mut Position) -> Move {
    // e2e4 e7e5 g1f3 b8c6 f1b5 c2c1q
    let from_file = move_string.chars().next().unwrap() as usize;
    let from_rank = move_string.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;
    let to_file = move_string.chars().nth(2).unwrap() as usize;
    let to_rank = move_string.chars().nth(3).unwrap().to_digit(10).unwrap() as usize;

    let prom_piece_str = move_string.chars().nth(4);
    let piece_color = match position.is_white_turn {
        true => WHITE,
        false => BLACK,
    };
    let prom_piece = match prom_piece_str {
        Some('n') => Some(KNIGHT | piece_color),
        Some('b') => Some(BISHOP | piece_color),
        Some('r') => Some(ROOK | piece_color),
        Some('q') => Some(QUEEN | piece_color),
        Some(_other) => None,
        None => None,
    };

    let from_square = (from_file - 97) + ((8 - from_rank) * 16);
    let to_square = (to_file - 97) + ((8 - to_rank) * 16);

    let moves = position.generate_legal_moves();
    for move_ in moves {
        if move_.from == from_square && move_.to == to_square && move_.promoted_piece == prom_piece
        {
            return move_;
        }
    }
    panic!(
        "Parsed move is not matched to any legal move: from: {} to: {}",
        from_square, to_square
    );
}

pub fn handle_position(input: &str, position: &mut Position) {
    // > position startpos
    // > position startpos moves e2e4 e7e5 g1f3 b8c6 f1b5
    // > position fen 8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1
    // > position fen 8/3P3k/n2K3p/2p3n1/1b4N1/2p1p1P1/8/3B4 w - - 0 1 moves g4f6 h7g7 f6h5 g7g6 d1c2
    if input.contains("fen") {
        let fen_part = input.strip_prefix("position fen ").unwrap();
        *position = Position::from_fen(fen_part);
    } else if input.contains("startpos") {
        *position = Position::from_fen(START_POSITION_FEN);
    }
    position.repetition_index += 1;
    position.repetition_stack[position.repetition_index] = position.hash;
    if input.contains("moves") {
        let index = input.find("moves").unwrap();
        let moves_part = &input[index + 6..];
        for move_string in moves_part.split(" ") {
            let move_ = parse_move(move_string, position);
            position.make_move(&move_, 0);
            position.repetition_index += 1;
            position.repetition_stack[position.repetition_index] = position.hash;
        }
        //println!("{moves_part}");
    }
}

fn handle_go(input: &str, position: &mut Position, tt: &mut TranspositionTable) {
    // default depth
    let mut depth: u32 = 64;

    let mut base: u64 = 0;
    let mut increment: u64 = 0;

    if input.contains("depth") {
        let index = input.find("depth").unwrap();
        depth = input[index + 6..].trim().parse::<u32>().unwrap();
    }
    if input.contains("wtime") && position.is_white_turn {
        let re = Regex::new(r"[\s\S]+wtime (\d+)").unwrap();
        let Some(caps) = re.captures(input) else {
            return;
        };
        base = caps[1].parse::<u64>().unwrap();
    }
    if input.contains("winc") && position.is_white_turn {
        let re = Regex::new(r"[\s\S]+winc (\d+)").unwrap();
        let Some(caps) = re.captures(input) else {
            return;
        };
        increment = caps[1].parse::<u64>().unwrap();
    }
    if input.contains("btime") && !position.is_white_turn {
        let re = Regex::new(r"[\s\S]+btime (\d+)").unwrap();
        let Some(caps) = re.captures(input) else {
            return;
        };
        base = caps[1].parse::<u64>().unwrap();
    }
    if input.contains("binc") && !position.is_white_turn {
        let re = Regex::new(r"[\s\S]+binc (\d+)").unwrap();
        let Some(caps) = re.captures(input) else {
            return;
        };
        increment = caps[1].parse::<u64>().unwrap();
    }
    let movetime: u64 = if input.contains("movetime") {
        let index = input.find("movetime").unwrap();
        input[index + 9..].trim().parse::<u64>().unwrap()
    } else {
        base / 20 + increment / 2
    };
    assert_ne!(
        movetime, 0,
        "movetime is zero, check that time command is for correct side"
    );

    let (pv, _node_count) = Search::run(position, tt, depth, movetime);
    let best_move = pv.first().expect("pv should have moves");
    println!("bestmove {}", get_move_string(best_move));
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
    let mut tt = TranspositionTable::new(64);

    let version = env!("CARGO_PKG_VERSION");
    let name = "Ratsu";

    println!("id name {name} {version}");
    println!("id author Eetu Rantala");
    println!("uciok");

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
            handle_go(&input, &mut position, &mut tt);
        } else if input.contains("perft") {
            // usage: perft 5
            let depth = input[6..].trim().parse::<u32>().unwrap();
            run_perft(depth, &mut position);
        } else if input.contains("stop") {
            println!("Handle stop")
            // stops calculating as soon as possible
        } else if input.contains("uci") {
            println!("id name {name} {version}");
            println!("id author Eetu Rantala");
            println!("uciok");
        }
    }
}
