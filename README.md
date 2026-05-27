# shanten_pyo

Rust/PyO3 shanten and yaku-distance feature module.

## Python API

`eval_hand_py(hand)` takes a 34-length tile count list and returns a 16-item tuple:

```python
(
    normal,
    chiitoi,
    kokushi,
    tanyao,
    (honitsu_man, honitsu_pin, honitsu_sou),
    yakuhai,
    pinfu,
    toitoi,
    (chinitsu_man, chinitsu_pin, chinitsu_sou),
    iipeko,
    sanshoku,
    ittsu,
    chanta,
    junchan,
    honroutou,
    shosangen,
)
```

`eval_discards_py(hand)` returns one 17-item tuple for each possible discard. The
first item is `tile_index`, followed by the same 16 metrics as `eval_hand_py`.

```python
import shanten_pyo

hand = [0] * 34
hand[1] = 3   # 222m
hand[4] = 1   # 5m
hand[5] = 1   # 6m
hand[6] = 1   # 7m
hand[11] = 1  # 3p
hand[12] = 1  # 4p
hand[13] = 1  # 5p
hand[19] = 2  # 22s
hand[23] = 1  # 6s
hand[24] = 2  # 77s

hand_metrics = shanten_pyo.eval_hand_py(hand)
discard_rows = shanten_pyo.eval_discards_py(hand)

assert len(hand_metrics) == 16
assert discard_rows
assert len(discard_rows[0]) == 17
```

Distances are proxy features for learning. Smaller values mean the hand is closer
to that yaku direction.
