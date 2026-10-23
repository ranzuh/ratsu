import chess
import chess.pgn
import re


def parse_eval(comment):
    """Parse eval from fastchess comment like '+12.54/10 0.022s'.
    Returns centipawns from the mover's perspective, or None (e.g. mate scores)."""
    match = re.search(r"([+-]?\d+\.?\d*)/(\d+)", comment)
    if match:
        return round(float(match.group(1)) * 100)
    return None


def extract_positions(pgn_path, output_path, skip_plies=0):
    results = []
    with open(pgn_path) as f:
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
                        results.append(f"{board.fen()} [{white_result}] [{white_score}]")

                board.push(move)

    with open(output_path, "w") as f:
        f.write("\n".join(results))

    print(f"Extracted {len(results)} positions")


extract_positions("/home/eetu/enginetesting/games.pgn", "data.txt", skip_plies=0)
