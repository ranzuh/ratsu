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
