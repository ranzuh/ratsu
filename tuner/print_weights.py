from tune import load_weights
import sys


def print_weights(weights):
    names = [
        "material_pawn",
        "material_knight",
        "material_bishop",
        "material_rook",
        "material_queen",
        "material_king",
        "doubled_pawn_penalty",
        "isolated_pawn_penalty",
        "backwards_pawn_penalty",
        "passed_pawn_bonus",
        "rook_semi_open_file_bonus",
        "rook_open_file_bonus",
        "rook_on_seventh_bonus",
        "bishop_pair_bonus",
    ]
    pst_names = [
        "pawn_mg_pst",
        "knight_mg_pst",
        "bishop_mg_pst",
        "rook_mg_pst",
        "queen_mg_pst",
        "king_mg_pst",
        "pawn_eg_pst",
        "knight_eg_pst",
        "bishop_eg_pst",
        "rook_eg_pst",
        "queen_eg_pst",
        "king_eg_pst",
    ]

    # scalars
    for i, name in enumerate(names):
        print(f"{name}: {weights[i]}")

    # PSTs
    for i, name in enumerate(pst_names):
        start = 14 + i * 64
        pst = weights[start : start + 64]
        print(f"\n{name}:")
        for row in range(8):
            row_vals = pst[row * 8 : (row + 1) * 8]
            print("  " + ", ".join(f"{v:4d}" for v in row_vals), end=",\n")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python print_weights.py <weights_file.json>")
        sys.exit(1)

    weights_file = sys.argv[1]
    weights = load_weights(weights_file)
    print_weights(weights)
