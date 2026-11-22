#!/usr/bin/env python3
"""Render the original looping 8-bit pinball soundtrack.

The tune is an original composition in the voice of an early-1980s AY-3-8910
PSG: two square-wave voices, a triangle bass and a noise drum kit. It is not a
recording from the 1984 game and contains no sampled material.

Run from app/:  python3 tools/generate_music.py
"""

from __future__ import annotations

import math
import wave
from pathlib import Path

import numpy as np

SR = 44_100
BPM = 126.0
BEAT = 60.0 / BPM
BARS = 8
OUT = Path(__file__).resolve().parent.parent / "assets" / "audio" / "music_theme.wav"


def midi(n: int) -> float:
    return 440.0 * 2.0 ** ((n - 69) / 12.0)


def add_tone(
    buf: np.ndarray,
    start: float,
    dur: float,
    f0: float,
    f1: float,
    amp: float,
    wave: str = "square",
) -> None:
    n = int(dur * SR)
    i0 = int(start * SR)
    if n <= 0 or i0 >= len(buf):
        return
    n = min(n, len(buf) - i0)
    p = np.linspace(0.0, 1.0, n, endpoint=False)
    freq = f0 * (f1 / f0) ** p if abs(f1 - f0) > 1e-6 else np.full(n, f0)
    phase = np.cumsum(freq) / SR
    if wave == "square":
        signal = np.where((phase % 1.0) < 0.5, 1.0, -1.0)
    elif wave == "triangle":
        signal = 4.0 * np.abs((phase % 1.0) - 0.5) - 1.0
    else:
        rng = np.random.default_rng(int(start * 1000) + n)
        signal = rng.choice([-1.0, 1.0], size=n)
    attack = np.clip(np.arange(n) / (0.004 * SR), 0.0, 1.0)
    env = attack * np.exp(-3.2 * p)
    buf[i0 : i0 + n] += signal * env * amp


def main() -> None:
    total = int(BARS * 4 * BEAT * SR) + 1
    buf = np.zeros(total, dtype=np.float64)

    chords = [
        [57, 60, 64],  # Am
        [53, 57, 60],  # F
        [60, 64, 67],  # C
        [55, 59, 62],  # G
    ]
    roots = [45, 41, 36, 43]
    melody = [
        [69, 72, 76, 72, 71, 69, 67, 64],
        [65, 69, 72, 69, 67, 65, 64, 62],
        [64, 67, 72, 76, 74, 72, 71, 67],
        [67, 71, 74, 71, 69, 67, 65, 62],
        [69, 72, 76, 79, 76, 72, 71, 69],
        [65, 69, 72, 77, 76, 72, 69, 65],
        [64, 67, 72, 76, 79, 76, 72, 67],
        [67, 71, 74, 79, 77, 74, 71, 67],
    ]

    for bar in range(BARS):
        start = bar * 4 * BEAT
        chord = chords[bar % 4]
        root = roots[bar % 4]

        # Lead square line.
        for step, note in enumerate(melody[bar]):
            t = start + step * BEAT * 0.5
            f = midi(note)
            add_tone(buf, t, BEAT * 0.43, f, f, 0.085, "square")

        # 16th-note arpeggio.
        for step in range(8):
            t = start + step * BEAT * 0.5
            for sub in range(2):
                note = chord[(step * 2 + sub) % 3] + 12
                f = midi(note)
                add_tone(buf, t + sub * BEAT * 0.25, BEAT * 0.20, f, f, 0.026, "square")

        # Triangle bass.
        for step in range(8):
            t = start + step * BEAT * 0.5
            note = root if step % 2 == 0 else root + 7
            f = midi(note)
            add_tone(buf, t, BEAT * 0.38, f, f, 0.075, "triangle")

        # Drums.
        for beat_i in range(4):
            t = start + beat_i * BEAT
            if beat_i % 2 == 0:
                add_tone(buf, t, 0.13, 120.0, 46.0, 0.11, "triangle")
            else:
                add_tone(buf, t, 0.10, 1800.0, 700.0, 0.055, "noise")
            add_tone(buf, t + BEAT * 0.5, 0.035, 6000.0, 5000.0, 0.018, "noise")

    # Keep the loop tidy and seamless.
    fade = int(0.008 * SR)
    buf[:fade] *= np.linspace(0.0, 1.0, fade)
    buf[-fade:] *= np.linspace(1.0, 0.0, fade)
    peak = np.max(np.abs(buf)) or 1.0
    buf *= 0.62 / peak
    pcm = np.clip(buf, -1.0, 1.0)
    pcm = (pcm * 32767.0).astype(np.int16)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(OUT), "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(SR)
        wav.writeframes(pcm.tobytes())
    print(f"wrote {OUT} ({len(pcm) / SR:.2f}s, {BPM:.0f} BPM, 8 bars)")


if __name__ == "__main__":
    main()
