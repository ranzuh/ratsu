## Generating training data with self-play

```
./fastchess \
    -engine cmd=./engines/ratsu-120 name=ratsu-120a \
    -engine cmd=./engines/ratsu-120 name=ratsu-120b \
    -openings file=./noob_5moves.epd format=epd order=random \
    -each tc=4+0.04 \
    -rounds 1000 -concurrency 12 \
    -pgnout file=./games.pgn
```