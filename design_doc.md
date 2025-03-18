Piece coding

00000    empty

white pieces:

01001    pawn
01010    knight
01011    bishop
01100    rook
01101    queen
01110    king

black pieces:

10001    pawn
10010    knight
10011    bishop
10100    rook
10101    queen
10110    king


reading FEN string
rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

split on " "

first part

"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"

read char by char
if char / move i+8

if char numeric move i+value

if char letter add piece with that letter

needed: char to piece map


Move generation

side to move

following pieces move symmetrically with both sides:
sliding pieces: bishop, rook, queen
knight, king

pawns move different direction, need to take into account

need to handle castling - different for both sides

same for en passant


Castling

1. check castling rights
2. check whether squares between king and rook are empty
3. check are squares that king passes attacked
4. create a move with king and is_castling flag
5. in makemove check is_castling flag and move the correct rook next to king based on move_.to


Castling rights
1. if king moves from starting square -> cancel castling rights
2. if a rook moves from starting square -> cancel castling rights for that side
3. if a rook get captured -> cancel castling rights for that side


En passant 

Make:
    player1 makes double pawn move (store in movegen) (read in make_move)
        check if player2 has pawn in either square for enpassant (in make_move?)
        if has then store the square behind player1 pawn as enpassant in next position (in make_move?)

    if there is enpassant square in position and player has pawn in correct rank (in movegen)
        add pawn capture with enpassant flag for each pawn that are in position to do en passant (max 2)

    if position has enpassant square (in make_move) remove it from the next position

Unmake:
    enpassant square needs to be same that it was

