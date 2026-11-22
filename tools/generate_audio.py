#!/usr/bin/env python3
"""Render the shipped ROLLERBALL sound banks.

The sounds follow the voice of an MSX AY-3-8910 PSG: three square-wave tone
channels, one noise channel, 16-step volume envelopes, fast pitch sweeps and
short arpeggios.  They are original recreations in that style, not samples
from the 1984 game.

Run from app/:  python3 tools/generate_audio.py
"""

from __future__ import annotations

import math
import subprocess
from pathlib import Path

import numpy as np

SR = 44_100
OUT = Path(__file__).resolve().parent.parent / "assets" / "audio"
rng = np.random.default_rng(1984)


# --------------------------------------------------------------------------
# PSG-ish building blocks
# --------------------------------------------------------------------------

def _freq_curve(f0: float, f1: float, n: int) -> np.ndarray:
    if abs(f1 - f0) < 1e-6:
        return np.full(n, f0)
    # Exponential sweeps sound even, like a PSG pitch slide.
    return f0 * (f1 / f0) ** (np.linspace(0.0, 1.0, n))


def _envelope(n: int, attack: float, decay: float, hold: float) -> np.ndarray:
    t = np.arange(n) / SR
    a = max(attack, 1.0 / SR)
    env = np.clip(t / a, 0.0, 1.0)
    held = np.minimum(t, max(hold, 0.0))
    decayed = np.exp(-decay * np.maximum(t - max(hold, 0.0), 0.0))
    env = env * np.where(t <= max(hold, 0.0), 1.0, decayed)
    # AY volume is 16 discrete steps: quantising adds the gritty PSG grain.
    return np.round(env * 15.0) / 15.0


def note(
    buf: np.ndarray,
    start: float,
    dur: float,
    f0: float,
    f1: float | None = None,
    amp: float = 0.2,
    *,
    decay: float = 3.0,
    vibrato: float = 0.0,
    vibrato_rate: float = 6.0,
    attack: float = 0.003,
    hold: float = 0.0,
    tremolo: float = 0.0,
    tremolo_rate: float = 9.0,
    detune: float = 0.0,
    noise_mix: float = 0.0,
    quantise: bool = True,
) -> None:
    """Add one PSG tone (optionally with its noise channel) to a mono buffer."""
    n = int(dur * SR)
    if n <= 0:
        return
    f1 = f0 if f1 is None else f1
    freqs = _freq_curve(f0, f1, n)
    t = np.arange(n) / SR
    if vibrato:
        freqs = freqs * 2.0 ** (vibrato / 12.0 * np.sin(2 * math.pi * vibrato_rate * t))
    if detune:
        freqs = freqs * 2.0 ** (detune / 1200.0)
    phase = np.cumsum(freqs) / SR
    square = np.where((phase % 1.0) < 0.5, 1.0, -1.0)
    env = _envelope(n, attack, decay, hold)
    if tremolo:
        env = env * (0.65 + 0.35 * (0.5 + 0.5 * np.sin(2 * math.pi * tremolo_rate * t)))
    if quantise:
        env = np.round(env * 15.0) / 15.0
    wave = square * (1.0 - noise_mix) + _noise(n) * noise_mix
    _add(buf, start, n, wave * env * amp)


def noise_burst(
    buf: np.ndarray,
    start: float,
    dur: float,
    amp: float,
    *,
    decay: float = 8.0,
    hold_rate: int = 2,
    tone: float | None = None,
) -> None:
    n = int(dur * SR)
    if n <= 0:
        return
    if tone:
        # AY noise is a clocked LFSR: a coarse sample-and-hold approximates it.
        n2 = n
        bits = rng.integers(0, 2, n2 // hold_rate + 2) * 2 - 1
        sig = np.repeat(bits, hold_rate)[:n2]
        sig = np.round(np.sin(2 * math.pi * tone * np.arange(n2) / SR) * 6) / 6
    else:
        bits = rng.integers(0, 2, n // hold_rate + 2) * 2 - 1
        sig = np.repeat(bits, hold_rate)[:n]
    env = _envelope(n, 0.001, decay, 0.0)
    _add(buf, start, n, sig * env * amp)


def _noise(n: int) -> np.ndarray:
    return rng.integers(0, 2, n) * 2.0 - 1.0


def _add(buf: np.ndarray, start: float, n: int, sig: np.ndarray) -> None:
    begin = int(start * SR)
    if begin >= buf.size:
        return
    end = min(begin + n, buf.size)
    count = end - begin
    if count > 0:
        buf[begin:end] += sig[:count]


def _psg_echo(buf: np.ndarray, delay: float, gain: float) -> np.ndarray:
    """Short slap-back on a second PSG channel, a common MSX trick."""
    out = buf.copy()
    d = int(delay * SR)
    if 0 < d < buf.size:
        out[d:] += buf[:-d] * gain
    return out


def _finish(buf: np.ndarray, peak: float = 0.70) -> np.ndarray:
    buf = np.nan_to_num(buf)
    # Gentle saturation rounds the square edges a little.
    buf = np.tanh(buf * 1.1) / 1.1
    m = float(np.max(np.abs(buf)))
    if m > 0:
        buf = buf * (peak / m)
    fade = min(int(0.006 * SR), buf.size)
    if fade:
        buf[-fade:] *= np.linspace(1.0, 0.0, fade)
    return buf


def render(seconds: float) -> np.ndarray:
    return np.zeros(int(seconds * SR), dtype=np.float64)


# --------------------------------------------------------------------------
# Individual effects
# --------------------------------------------------------------------------

def start() -> np.ndarray:
    b = render(1.25)
    c5, e5, g5, c6 = 523.25, 659.25, 783.99, 1046.50
    for i, f in enumerate((c5, e5, g5, c6)):
        note(b, i * 0.085, 0.10, f, amp=0.20, decay=4.0, detune=5)
    note(b, 0.34, 0.70, c6, amp=0.13, decay=1.8, hold=0.30, vibrato=3.0)
    note(b, 0.34, 0.70, e5 * 2, amp=0.09, decay=1.8, hold=0.30)
    note(b, 0.34, 0.70, g5 * 2, amp=0.08, decay=1.8, hold=0.30)
    note(b, 0.34, 0.70, c5 / 2, amp=0.17, decay=2.2, hold=0.25)
    for i in range(4):
        noise_burst(b, 0.34 + i * 0.10, 0.03, 0.05, decay=12.0)
    return _finish(b)


def plunger_charge() -> np.ndarray:
    b = render(0.98)
    note(b, 0.0, 0.96, 98.0, 392.0, 0.16, decay=0.7, hold=0.9, tremolo=0.4,
         tremolo_rate=14.0)
    note(b, 0.0, 0.96, 196.0, 784.0, 0.07, decay=0.7, hold=0.9)
    # Ratchet ticks climb with the spring.
    for i in range(22):
        t = 0.03 + i * 0.041
        f = 240.0 + i * 34.0
        note(b, t, 0.018, f, amp=0.05 + 0.003 * i, decay=14.0)
        if i % 3 == 0:
            noise_burst(b, t, 0.012, 0.03 + 0.001 * i, decay=18.0)
    return _finish(b)


def plunger_release() -> np.ndarray:
    b = render(0.34)
    note(b, 0.0, 0.17, 760.0, 90.0, 0.26, decay=3.0)
    note(b, 0.0, 0.09, 1500.0, 420.0, 0.08, decay=5.0)
    noise_burst(b, 0.0, 0.07, 0.18, decay=9.0)
    note(b, 0.02, 0.26, 95.0, 55.0, 0.24, decay=5.0, hold=0.02)
    return _finish(_psg_echo(b, 0.035, 0.18))


def flipper_hit() -> np.ndarray:
    b = render(0.12)
    noise_burst(b, 0.0, 0.022, 0.20, decay=16.0)
    note(b, 0.0, 0.07, 300.0, 180.0, 0.25, decay=7.0)
    note(b, 0.005, 0.05, 610.0, 360.0, 0.09, decay=9.0)
    note(b, 0.01, 0.06, 120.0, 96.0, 0.10, decay=8.0)
    return _finish(b)


def flipper_hit_2() -> np.ndarray:
    b = render(0.12)
    noise_burst(b, 0.0, 0.020, 0.18, decay=18.0, hold_rate=3)
    note(b, 0.0, 0.075, 262.0, 150.0, 0.23, decay=6.5, detune=-9)
    note(b, 0.006, 0.05, 520.0, 300.0, 0.10, decay=8.0)
    note(b, 0.012, 0.06, 104.0, 82.0, 0.10, decay=8.0)
    return _finish(b)


def bumper_hit() -> np.ndarray:
    b = render(0.22)
    noise_burst(b, 0.0, 0.016, 0.11, decay=20.0)
    note(b, 0.0, 0.18, 1046.5, 784.0, 0.24, decay=4.2, vibrato=4.0,
         vibrato_rate=11.0)
    note(b, 0.005, 0.09, 2093.0, 1568.0, 0.10, decay=6.0)
    note(b, 0.0, 0.10, 523.25, 392.0, 0.10, decay=5.0)
    return _finish(_psg_echo(b, 0.045, 0.16))


def bumper_hit_2() -> np.ndarray:
    b = render(0.22)
    noise_burst(b, 0.0, 0.016, 0.10, decay=20.0, hold_rate=3)
    note(b, 0.0, 0.18, 1318.5, 988.0, 0.23, decay=4.0, vibrato=4.0,
         vibrato_rate=12.0)
    note(b, 0.006, 0.09, 2637.0, 1975.5, 0.09, decay=6.0)
    note(b, 0.0, 0.10, 659.25, 493.9, 0.09, decay=5.0)
    return _finish(_psg_echo(b, 0.040, 0.15))


def bumper_hit_3() -> np.ndarray:
    b = render(0.24)
    noise_burst(b, 0.0, 0.018, 0.10, decay=18.0)
    note(b, 0.0, 0.19, 1568.0, 1174.7, 0.22, decay=3.8, vibrato=5.0,
         vibrato_rate=10.0)
    note(b, 0.006, 0.10, 3136.0, 2349.3, 0.08, decay=5.5)
    note(b, 0.0, 0.11, 784.0, 587.3, 0.09, decay=4.5)
    return _finish(_psg_echo(b, 0.050, 0.17))


def rollover_lane() -> np.ndarray:
    b = render(0.18)
    note(b, 0.0, 0.055, 1318.5, amp=0.19, decay=7.0)
    note(b, 0.055, 0.10, 1760.0, amp=0.20, decay=6.0, detune=4)
    noise_burst(b, 0.0, 0.012, 0.05, decay=20.0)
    return _finish(b)


def rollover_lane_2() -> np.ndarray:
    b = render(0.18)
    note(b, 0.0, 0.055, 1760.0, amp=0.18, decay=7.0)
    note(b, 0.055, 0.10, 1318.5, amp=0.20, decay=6.0)
    noise_burst(b, 0.0, 0.012, 0.05, decay=20.0, hold_rate=3)
    return _finish(b)


def slot_spin() -> np.ndarray:
    b = render(0.92)
    for i in range(10):
        t = i * 0.073
        f = 620.0 + i * 74.0
        note(b, t, 0.026, f, amp=0.12, decay=13.0)
        noise_burst(b, t, 0.014, 0.06, decay=20.0)
    note(b, 0.76, 0.12, 180.0, 95.0, 0.20, decay=6.0)
    noise_burst(b, 0.76, 0.035, 0.10, decay=14.0)
    return _finish(b)


def slot_win() -> np.ndarray:
    b = render(1.00)
    notes = (523.25, 659.25, 783.99, 1046.5, 1318.5, 1568.0)
    for i, f in enumerate(notes):
        note(b, i * 0.068, 0.075, f, amp=0.17, decay=5.0, detune=6)
    for i, f in enumerate((1046.5, 1318.5, 1568.0)):
        note(b, 0.44, 0.54, f, amp=0.10, decay=1.9, hold=0.24, vibrato=3.0)
    note(b, 0.44, 0.54, 523.25 / 2, amp=0.16, decay=2.2, hold=0.20)
    for i in range(5):
        noise_burst(b, 0.44 + i * 0.09, 0.025, 0.06, decay=16.0)
    return _finish(b)


def chute_drop() -> np.ndarray:
    b = render(0.52)
    note(b, 0.0, 0.40, 1200.0, 190.0, 0.18, decay=1.8, vibrato=4.0,
         vibrato_rate=7.0)
    note(b, 0.0, 0.40, 600.0, 95.0, 0.08, decay=1.8)
    note(b, 0.24, 0.24, 120.0, 58.0, 0.13, decay=3.5)
    noise_burst(b, 0.0, 0.22, 0.05, decay=5.0, hold_rate=4)
    return _finish(b)


def bonus_tick() -> np.ndarray:
    b = render(0.055)
    note(b, 0.0, 0.04, 1760.0, amp=0.18, decay=11.0)
    note(b, 0.0, 0.03, 880.0, amp=0.08, decay=12.0)
    noise_burst(b, 0.0, 0.008, 0.05, decay=22.0)
    return _finish(b)


def ball_drain() -> np.ndarray:
    b = render(0.90)
    note(b, 0.0, 0.78, 440.0, 70.0, 0.21, decay=1.3, vibrato=5.0,
         vibrato_rate=5.0)
    note(b, 0.0, 0.78, 220.0, 40.0, 0.11, decay=1.6)
    noise_burst(b, 0.0, 0.16, 0.08, decay=6.0)
    note(b, 0.55, 0.30, 72.0, 46.0, 0.22, decay=4.0)
    return _finish(_psg_echo(b, 0.075, 0.16))


def game_over() -> np.ndarray:
    b = render(1.55)
    for i, f in enumerate((440.0, 349.23, 293.66)):
        note(b, i * 0.18, 0.20, f, amp=0.19, decay=3.6, detune=4)
    note(b, 0.54, 0.34, 220.0, amp=0.12, decay=3.0)
    for f in (146.83, 293.66):
        note(b, 0.86, 0.62, f, amp=0.15 if f < 200 else 0.09, decay=2.0,
             hold=0.22, vibrato=3.5)
    noise_burst(b, 0.86, 0.05, 0.05, decay=10.0)
    return _finish(b)


def kickback() -> np.ndarray:
    b = render(0.40)
    note(b, 0.0, 0.22, 105.0, 58.0, 0.30, decay=5.0)
    note(b, 0.0, 0.12, 210.0, 700.0, 0.16, decay=5.0)
    noise_burst(b, 0.0, 0.13, 0.20, decay=8.0)
    note(b, 0.02, 0.20, 420.0, 160.0, 0.10, decay=4.0)
    return _finish(_psg_echo(b, 0.04, 0.16))


def extra_ball() -> np.ndarray:
    b = render(0.95)
    for i, f in enumerate((659.25, 783.99, 987.77, 1318.5)):
        note(b, i * 0.082, 0.10, f, amp=0.18, decay=4.5, detune=5)
    for f in (1318.5, 1568.0, 1975.5):
        note(b, 0.42, 0.42, f, amp=0.09, decay=2.2, hold=0.18, vibrato=2.5)
    note(b, 0.42, 0.42, 329.63, amp=0.16, decay=2.4, hold=0.16)
    for i in range(4):
        noise_burst(b, 0.42 + i * 0.085, 0.022, 0.05, decay=16.0)
    return _finish(b)


def tilt() -> np.ndarray:
    b = render(0.75)
    for i in range(14):
        f = 116.0 if i % 2 == 0 else 104.0
        note(b, i * 0.048, 0.045, f, amp=0.16, decay=2.0, hold=0.03)
        note(b, i * 0.048, 0.045, f * 2, amp=0.06, decay=2.0, hold=0.03)
    noise_burst(b, 0.0, 0.65, 0.035, decay=0.9, hold_rate=6)
    note(b, 0.0, 0.68, 58.0, amp=0.10, decay=0.8, hold=0.5)
    return _finish(b)


# --------------------------------------------------------------------------
# Write the bank
# --------------------------------------------------------------------------

RECIPES = {
    "start": start,
    "plunger_charge": plunger_charge,
    "plunger_release": plunger_release,
    "flipper_hit": flipper_hit,
    "flipper_hit_2": flipper_hit_2,
    "bumper_hit": bumper_hit,
    "bumper_hit_2": bumper_hit_2,
    "bumper_hit_3": bumper_hit_3,
    "rollover_lane": rollover_lane,
    "rollover_lane_2": rollover_lane_2,
    "slot_spin": slot_spin,
    "slot_win": slot_win,
    "chute_drop": chute_drop,
    "bonus_countdown_tick": bonus_tick,
    "ball_drain": ball_drain,
    "game_over": game_over,
    "kickback": kickback,
    "extra_ball": extra_ball,
    "tilt": tilt,
}


def write_wav(name: str, samples: np.ndarray) -> Path:
    import wave

    path = OUT / f"{name}.wav"
    pcm = np.clip(samples * 32767.0, -32768, 32767).astype("<i2")
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(pcm.tobytes())
    return path


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    peaks = {}
    for name, recipe in RECIPES.items():
        samples = recipe()
        path = write_wav(name, samples)
        peaks[name] = (len(samples) / SR, float(np.max(np.abs(samples))))
        print(f"{path.name:28s} {peaks[name][0]:5.2f}s peak {peaks[name][1]:.2f}")

    # Keep one OGG in the bank so both supported formats ship.
    wav = OUT / "slot_win.wav"
    ogg = OUT / "slot_win.ogg"
    subprocess.run(
        ["ffmpeg", "-y", "-loglevel", "error", "-i", str(wav),
         "-c:a", "libvorbis", "-q:a", "5", str(ogg)],
        check=True,
    )
    wav.unlink()
    print(f"{ogg.name:28s} {ogg.stat().st_size / 1024:.0f} KB")


if __name__ == "__main__":
    main()
