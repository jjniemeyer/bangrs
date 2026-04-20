#!/usr/bin/env bash
# crates/bangrs-scan/tests/fixtures/gen-fixtures.sh
# Requires: sox, ffmpeg, id3v2
set -euo pipefail
cd "$(dirname "$0")"

# silence-1s.wav — stereo 44.1kHz, full ID3 tags
sox -n -r 44100 -c 2 silence-1s.wav trim 0 1

# sine-440-1s.mp3 — 440Hz sine, 1s, ID3v2 tags
sox -n -r 44100 -c 2 sine-440.wav synth 1 sine 440
ffmpeg -y -i sine-440.wav -metadata title="Sine 440" -metadata artist="TestArtist" -metadata album="TestAlbum" sine-440-1s.mp3
rm sine-440.wav

# sine-880-1s.flac — 880Hz sine, Vorbis comments
sox -n -r 44100 -c 2 sine-880.wav synth 1 sine 880
ffmpeg -y -i sine-880.wav -metadata title="Sine 880" -metadata artist="TestArtist" sine-880-1s.flac
rm sine-880.wav

# tagless.wav — no tags
sox -n -r 44100 -c 2 tagless.wav synth 0.5 sine 220

# corrupt-tags.mp3 — valid MP3 with an intentionally broken ID3v2 frame
ffmpeg -y -f lavfi -i "sine=frequency=330:duration=1" -metadata title="Intact Tag" corrupt-tags.mp3
# Overwrite the first few bytes of the ID3 header with garbage to corrupt it
printf '\x01\x02\x03\x04' | dd of=corrupt-tags.mp3 bs=1 seek=3 count=4 conv=notrunc 2>/dev/null
