use crate::{Position, movegen::print_move};

pub fn perft(depth: u32, position: &mut Position) -> u64 {
    let mut nodes: u64 = 0;

    if depth == 0 {
        return 1;
    }

    let moves = position.generate_moves();
    for move_ in moves {
        let target_piece = position.board[move_.to];
        position.make_move(&move_);

        nodes += perft(depth - 1, position);

        position.unmake_move(&move_, target_piece);
    }
    nodes
}
