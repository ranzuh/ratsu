use crate::{Position, movegen::print_move};

// TODO: remove later? - debug stuff
pub static mut CASTLINGS: i32 = 0;
pub static mut CAPTURES: i32 = 0;
pub static mut ENPASSANTS: i32 = 0;
pub static mut PROMOTIONS: i32 = 0;

pub unsafe fn perft(depth: u32, position: &mut Position) -> u64 {
    let mut nodes: u64 = 0;

    if depth == 0 {
        return 1;
    }

    let moves = position.generate_legal_moves();
    for move_ in moves {
        // TODO: remove later? - debug stuff
        if move_.is_castling {
            CASTLINGS += 1;
        }
        if move_.is_capture {
            CAPTURES += 1;
        }
        if move_.is_enpassant {
            ENPASSANTS += 1;
        }
        if move_.promoted_piece.is_some() {
            PROMOTIONS += 1;
        }

        let target_piece = position.board[move_.to];
        let original_castling_rights = position.castling_rights;
        let original_king_squares = position.king_squares;
        position.make_move(&move_);

        nodes += perft(depth - 1, position);

        position.unmake_move(
            &move_,
            target_piece,
            original_castling_rights,
            original_king_squares,
        );
    }
    nodes
}
