struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
}

pub struct ZobristKeys {
    pub piece_keys: [[u64; 24]; 64],
    pub black_to_move_key: u64,
    pub castling_rights_keys: [u64; 4],
    pub enpassant_file_keys: [u64; 8],
}

impl ZobristKeys {
    pub fn new() -> Self {
        const SEED: u64 = 0x1EF105C43DEF1F9F;
        let mut state = Xorshift64::new(SEED);

        Self {
            piece_keys: std::array::from_fn(|_| std::array::from_fn(|_| state.next())),
            black_to_move_key: state.next(),
            castling_rights_keys: std::array::from_fn(|_| state.next()),
            enpassant_file_keys: std::array::from_fn(|_| state.next()),
        }
    }
}
