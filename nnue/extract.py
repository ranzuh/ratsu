import chess
import chess.pgn
import re
from functools import partial
import io
from multiprocessing.pool import Pool
import time


def parse_eval(comment):
    """Parse eval from fastchess comment like '+12.54/10 0.022s'.
    Returns centipawns from the mover's perspective, or None (e.g. mate scores)."""
    match = re.match(r"([+-]?\d+\.?\d*)/(\d+)", comment)
    if match:
        return round(float(match.group(1)) * 100)
    return None


def process_game(game_text, skip_plies=0):
    game = chess.pgn.read_game(io.StringIO(game_text))

    result_str = game.headers.get("Result", "*")
    if result_str == "1-0":
        white_result = 1.0
    elif result_str == "0-1":
        white_result = 0.0
    elif result_str == "1/2-1/2":
        white_result = 0.5
    else:
        return []

    training_lines = []
    board = game.board()
    for ply, node in enumerate(game.mainline()):
        move = node.move

        # board is the position BEFORE the move: the one we'd store
        keep = (
            ply >= skip_plies
            and not board.is_check()        # stm not in check
            and not board.is_capture(move)  # best move not a capture
        )

        if keep:
            score = parse_eval(node.comment)
            if score is not None and abs(score) <= 30000:
                white_score = score if board.turn == chess.WHITE else -score
                training_lines.append(f"{board.fen()} [{white_result}] [{white_score}]")

        board.push(move)

    return training_lines

def read_game_texts(path):
    buf = []
    with open(path) as f:
        for line in f:
            if line.startswith("[Event ") and buf:
                yield "".join(buf)
                buf = []
            buf.append(line)
    if buf:
        yield "".join(buf)

if __name__ == '__main__':
    start = time.time()
    worker = partial(process_game, skip_plies=0)
    count = 0

    with Pool() as pool, open("data.txt", "w") as out:
        for lines in pool.imap_unordered(
            worker, read_game_texts("/Users/eetu/enginetesting/games.pgn"), chunksize=64
        ):
            for line in lines:
                out.write(line + "\n")
            count += len(lines)

    end = time.time()
    print(f"Extracted {count} positions in {end - start:.2f}s")
