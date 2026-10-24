import io
import re
from multiprocessing import Pool

import chess
import chess.pgn

EVAL_RE = re.compile(r"([+-]?\d+\.?\d*)/\d+")


def parse_eval(comment):
    """Parse eval from fastchess comment like '+12.54/10 0.022s'.
    Returns centipawns from the mover's perspective, or None (e.g. mate scores)."""
    match = EVAL_RE.search(comment)
    if match:
        return round(float(match.group(1)) * 100)
    return None


def process_chunk(args):
    text, skip_plies = args
    results = []
    f = io.StringIO(text)
    while True:
        game = chess.pgn.read_game(f)
        if game is None:
            break

        result_str = game.headers.get("Result", "*")
        if result_str == "1-0":
            white_result = 1.0
        elif result_str == "0-1":
            white_result = 0.0
        elif result_str == "1/2-1/2":
            white_result = 0.5
        else:
            continue

        board = game.board()
        for ply, node in enumerate(game.mainline()):
            move = node.move

            if ply >= skip_plies:
                score = parse_eval(node.comment)  # cheap check first
                if (
                    score is not None
                    and abs(score) <= 30000
                    and not board.is_check()
                    and not board.is_capture(move)
                ):
                    white_score = score if board.turn == chess.WHITE else -score
                    results.append(f"{board.fen()} [{white_result}] [{white_score}]")

            board.push(move)
    return results


def read_chunks(path, skip_plies, games_per_chunk=200):
    """Yield (pgn_text, skip_plies) chunks split at game boundaries."""
    chunk, n = [], 0
    with open(path) as f:
        for line in f:
            if line.startswith("[Event "):
                if n == games_per_chunk:
                    yield "".join(chunk), skip_plies
                    chunk, n = [], 0
                n += 1
            chunk.append(line)
    if chunk:
        yield "".join(chunk), skip_plies


def extract_positions(pgn_path, output_path, skip_plies=0):
    total = 0
    with Pool() as pool, open(output_path, "w") as out:
        for results in pool.imap(process_chunk, read_chunks(pgn_path, skip_plies)):
            if results:
                out.write("\n".join(results) + "\n")
                total += len(results)
    print(f"Extracted {total} positions")


if __name__ == "__main__":
    extract_positions("/home/eetu/enginetesting/games-d6-10.pgn", "data-d6-more-more5.txt", skip_plies=0)