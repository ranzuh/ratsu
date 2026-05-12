import chess
import chess.pgn

import re

def parse_eval(comment):
    """Parse eval from fastchess comment like '+12.54/10 0.022s'"""
    match = re.search(r'([+-]?\d+\.?\d*)/(\d+)', comment)
    if match:
        score = float(match.group(1))
        return int(score * 100)  # convert to centipawns
    return None

def extract_positions(pgn_path, output_path, skip_plies=16):
    results = []
    with open(pgn_path) as f:
        while True:
            game = chess.pgn.read_game(f)
            if game is None:
                break
            
            result_str = game.headers.get("Result", "*")
            if result_str == "1-0":
                result = 1.0
            elif result_str == "0-1":
                result = 0.0
            elif result_str == "1/2-1/2":
                result = 0.5
            else:
                continue  # skip unfinished games
            
            board = game.board()
            for ply, node in enumerate(game.mainline()):
                move = node.move
                # Filter BEFORE pushing
                if ply < skip_plies:
                    board.push(move)
                    continue
                if board.is_capture(move):
                    board.push(move)
                    continue

                is_white = node.turn()
                score = parse_eval(node.comment)
                if is_white: score *= -1
                
                if score is not None and abs(score) > 30000:
                    board.push(move)
                    continue
                
                board.push(move)
                
                if board.is_check():
                    continue
                
                fen = board.fen()
                score_str = str(score) if score is not None else "none"
                results.append(f"{fen} [{result}] [{score_str}]")
                
    
    with open(output_path, 'w') as f:
        f.write('\n'.join(results))
    
    print(f"Extracted {len(results)} positions")

extract_positions('games.pgn', 'data.txt')
