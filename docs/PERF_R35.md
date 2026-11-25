# R35 Performance Log (`tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland`)

## Before (R34 baseline)

| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |
|---|---|---|---|---|---|
| camp | 137.6 | 33.7 | 136.8 | 9.9 | 58 (24) |
| forest | 106.3 | 45.0 | 165.8 | 12.9 | 71 (19) |
| city | 103.4 | 31.7 | 241.7 | 15.3 | 61 (26) |
| night | 127.3 | 37.9 | 158.4 | 10.9 | 60 (0) |
| zombies | 93.8 | 32.7 | 106.4 | 16.6 | 51 (18) |
| desert | 162.6 | 44.6 | 138.9 | 10.2 | 39 (14) |
| snow | 133.4 | 31.3 | 148.8 | 13.0 | 48 (18) |
| swamp | 141.8 | 42.4 | 144.6 | 11.8 | 49 (20) |
| mountains | 131.5 | 44.2 | 164.9 | 12.0 | 45 (19) |
| town_desert | 125.0 | 42.4 | 138.7 | 12.6 | 50 (20) |
| town_snow | 115.5 | 43.8 | 249.3 | 13.9 | 47 (23) |

## After (R35 all-out detail, tuned `data/ground_detail.json`)

| Scene | Avg FPS | 1% Low FPS | Worst Frame (ms) | p95 (ms) | Draws (Shadow) |
|---|---|---|---|---|---|
| camp | 123.4 | 47.6 | 115.9 | 12.1 | 60 (24) |
| forest | 86.9 | 34.1 | 155.7 | 15.7 | 71 (19) |
| city | 97.7 | 34.2 | 166.9 | 16.2 | 62 (26) |
| night | 128.0 | 56.4 | 154.2 | 11.4 | 61 (0) |
| zombies | 68.7 | 25.7 | 109.4 | 21.3 | 52 (18) |
| desert | 121.7 | 34.6 | 216.5 | 14.2 | 40 (14) |
| snow | 113.1 | 36.3 | 148.7 | 15.7 | 46 (19) |
| swamp | 93.0 | 27.7 | 172.3 | 18.7 | 52 (21) |
| mountains | 97.9 | 29.1 | 194.3 | 16.9 | 48 (19) |
| town_desert | 107.7 | 37.9 | 186.0 | 15.4 | 65 (33) |
| town_snow | 75.3 | 26.5 | 233.2 | 21.9 | 59 (33) |
