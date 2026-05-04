pub const NOT_A: u64 = !0x0101010101010101;
pub const NOT_H: u64 = !0x8080808080808080;
pub const RANK_7: u64 = 0x00FF000000000000; // rank 7 (black's 2nd rank)
pub const RANK_2: u64 = 0x000000000000FF00; // rank 2 (white's 2nd rank)

#[rustfmt::skip]
#[repr(u8)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

pub fn print_bb(bb: u64) {
    for rank in (0..8).rev() {
        for file in 0..8 {
            let square = rank * 8 + file;
            let bit = 1u64 << square;
            if bb & bit != 0 {
                print!("1 ");
            } else {
                print!(". ");
            }
        }
        println!();
    }
    println!();
}

pub fn south_fill(mut bb: u64) -> u64 {
    bb |= bb >> 8;
    bb |= bb >> 16;
    bb |= bb >> 32;
    bb
}

pub fn north_fill(mut bb: u64) -> u64 {
    bb |= bb << 8;
    bb |= bb << 16;
    bb |= bb << 32;
    bb
}

pub fn file_fill(bb: u64) -> u64 {
    north_fill(south_fill(bb))
}

pub fn adjacent_files(bb: u64) -> u64 {
    ((bb & NOT_A) >> 1) | ((bb & NOT_H) << 1)
}
