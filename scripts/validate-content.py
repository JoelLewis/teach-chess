#!/usr/bin/env python3
"""Validate puzzle CSV and opening JSON content files."""

from __future__ import annotations

import csv
import json
import re
import sys
from pathlib import Path

PUZZLE_REQUIRED_COLUMNS = {"PuzzleId", "FEN", "Moves", "Rating", "Themes"}
OPENING_REQUIRED_FIELDS = {"name", "eco", "color", "description", "moves", "themes", "difficulty"}
FEN_PATTERN = re.compile(r"^[rnbqkpRNBQKP1-8/]+ [wb] [KQkq-]+ [a-h1-8-]+ \d+ \d+$")
UCI_MOVE_PATTERN = re.compile(r"^[a-h][1-8][a-h][1-8][qrbn]?$")


def validate_puzzles(data_dir: Path) -> list[str]:
    errors: list[str] = []
    csv_files = list(data_dir.glob("puzzles*.csv"))

    if not csv_files:
        errors.append("No puzzle CSV files found")
        return errors

    for csv_path in csv_files:
        with open(csv_path, newline="", encoding="utf-8") as f:
            reader = csv.DictReader(f)
            if reader.fieldnames is None:
                errors.append(f"{csv_path.name}: empty or missing header")
                continue

            missing = PUZZLE_REQUIRED_COLUMNS - set(reader.fieldnames)
            if missing:
                errors.append(f"{csv_path.name}: missing columns: {missing}")
                continue

            for i, row in enumerate(reader, start=2):
                puzzle_id = row.get("PuzzleId", "")
                fen = row.get("FEN", "")
                moves = row.get("Moves", "")
                rating = row.get("Rating", "")

                if not puzzle_id.strip():
                    errors.append(f"{csv_path.name}:{i}: empty PuzzleId")

                if not FEN_PATTERN.match(fen):
                    errors.append(f"{csv_path.name}:{i}: invalid FEN: {fen[:60]}")

                if not moves.strip():
                    errors.append(f"{csv_path.name}:{i}: empty Moves")
                else:
                    for move in moves.strip().split():
                        if not UCI_MOVE_PATTERN.match(move):
                            errors.append(f"{csv_path.name}:{i}: invalid UCI move: {move}")
                            break

                if not rating.strip().isdigit():
                    errors.append(f"{csv_path.name}:{i}: invalid Rating: {rating}")
                elif not 0 <= int(rating) <= 4000:
                    errors.append(f"{csv_path.name}:{i}: rating out of range: {rating}")

        print(f"  Checked {csv_path.name}: {i - 1} puzzles")

    return errors


def validate_openings(data_dir: Path) -> list[str]:
    errors: list[str] = []
    json_files = list(data_dir.glob("openings*.json"))

    if not json_files:
        errors.append("No opening JSON files found")
        return errors

    for json_path in json_files:
        with open(json_path, encoding="utf-8") as f:
            try:
                data = json.load(f)
            except json.JSONDecodeError as e:
                errors.append(f"{json_path.name}: invalid JSON: {e}")
                continue

        if not isinstance(data, list):
            errors.append(f"{json_path.name}: top-level must be an array")
            continue

        for i, opening in enumerate(data):
            if not isinstance(opening, dict):
                errors.append(f"{json_path.name}[{i}]: must be an object")
                continue

            missing = OPENING_REQUIRED_FIELDS - set(opening.keys())
            if missing:
                errors.append(f"{json_path.name}[{i}] ({opening.get('name', '?')}): missing fields: {missing}")
                continue

            if opening["color"] not in ("white", "black"):
                errors.append(f"{json_path.name}[{i}] ({opening['name']}): color must be 'white' or 'black'")

            moves = opening["moves"].strip().split()
            for move in moves:
                if not UCI_MOVE_PATTERN.match(move):
                    errors.append(f"{json_path.name}[{i}] ({opening['name']}): invalid UCI move: {move}")
                    break

            difficulty = opening["difficulty"]
            if not isinstance(difficulty, int) or not 0 <= difficulty <= 4000:
                errors.append(f"{json_path.name}[{i}] ({opening['name']}): invalid difficulty: {difficulty}")

        print(f"  Checked {json_path.name}: {len(data)} openings")

    return errors


def main() -> None:
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <puzzles|openings> <data_dir>")
        sys.exit(1)

    content_type = sys.argv[1]
    data_dir = Path(sys.argv[2])

    if not data_dir.is_dir():
        print(f"Error: {data_dir} is not a directory")
        sys.exit(1)

    print(f"Validating {content_type} in {data_dir}/...")

    if content_type == "puzzles":
        errors = validate_puzzles(data_dir)
    elif content_type == "openings":
        errors = validate_openings(data_dir)
    else:
        print(f"Unknown content type: {content_type}")
        sys.exit(1)

    if errors:
        print(f"\n{len(errors)} validation error(s):")
        for err in errors:
            print(f"  - {err}")
        sys.exit(1)
    else:
        print("All validations passed.")


if __name__ == "__main__":
    main()
