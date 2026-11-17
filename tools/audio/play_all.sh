#!/usr/bin/env bash
# Plays every Rotland sound one after another, so the sounds can be heard in one go.
# Needs aplay (ALSA) or paplay (PulseAudio/PipeWire). Run it from the project folder:
#   tools/audio/play_all.sh
# To stop early, press Ctrl+C.
set -u

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

if command -v paplay >/dev/null 2>&1; then
    PLAY=(paplay)
elif command -v aplay >/dev/null 2>&1; then
    PLAY=(aplay -q)
else
    echo "No player found. Install one with: sudo apt install alsa-utils"
    exit 1
fi

shopt -s nullglob
for file in "$ROOT"/assets/audio/sfx/*.wav "$ROOT"/assets/audio/ambient/*.wav; do
    echo "Playing $(basename "$file")"
    "${PLAY[@]}" "$file"
    sleep 0.4
done
echo "Done."
