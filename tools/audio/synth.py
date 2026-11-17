#!/usr/bin/env python3
"""Synthesizes Rotland's v1 sound effects and ambience loops (no recordings, no downloads).

Writes 16-bit mono 44.1 kHz WAV files into assets/audio/sfx and assets/audio/ambient.
Every sound has a fixed seed, so running this again gives the same files.
Run from anywhere:  python3 tools/audio/synth.py
Needs Python 3 and numpy only. Prints each file with its duration and peak level.
"""
import os
import wave
import zlib

import numpy as np

SR = 44100
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SFX_DIR = os.path.join(ROOT, "assets", "audio", "sfx")
AMBIENT_DIR = os.path.join(ROOT, "assets", "audio", "ambient")
TARGET_PEAK = 10 ** (-3.0 / 20.0)  # -3 dBFS
LOOP_SECONDS = 20.0


# ---------- building blocks ----------

def rng_for(name):
    return np.random.default_rng(zlib.crc32(name.encode("utf-8")))


def times(n):
    return np.arange(n) / SR


def white(n, rng):
    return rng.standard_normal(n)


def pink(n, rng):
    spectrum = np.fft.rfft(white(n, rng))
    freqs = np.fft.rfftfreq(n, 1.0 / SR)
    freqs[0] = freqs[1]
    x = np.fft.irfft(spectrum / np.sqrt(freqs), n)
    return x / np.max(np.abs(x))


def brown(n, rng):
    # Built in the frequency domain (1/f amplitude). A cumulative sum would not repeat, so the loop would click.
    spectrum = np.fft.rfft(white(n, rng))
    freqs = np.fft.rfftfreq(n, 1.0 / SR)
    freqs[0] = freqs[1]
    x = np.fft.irfft(spectrum / freqs, n)
    return x / np.max(np.abs(x))


def filt(x, lo=None, hi=None):
    """Band filter done in the frequency domain (Butterworth-like magnitude, zero phase).
    It is a circular filter, so a periodic signal stays periodic: this is what makes the loops seamless."""
    n = len(x)
    spectrum = np.fft.rfft(x)
    freqs = np.fft.rfftfreq(n, 1.0 / SR)
    gain = np.ones_like(freqs)
    if lo:
        gain *= 1.0 / np.sqrt(1.0 + (lo / np.maximum(freqs, 1e-9)) ** 4)
    if hi:
        gain *= 1.0 / np.sqrt(1.0 + (freqs / hi) ** 4)
    return np.fft.irfft(spectrum * gain, n)


def glide(f0, f1, n):
    """Phase of a tone whose pitch glides exponentially from f0 to f1 over n samples."""
    t = times(n)
    freq = f0 * (f1 / f0) ** (t / max(n / SR, 1e-9))
    return 2.0 * np.pi * np.cumsum(freq) / SR


def tone(freq, n):
    return np.sin(2.0 * np.pi * freq * times(n))


def decay(n, tau):
    return np.exp(-times(n) / tau)


def attack(n, seconds):
    return np.minimum(1.0, times(n) / max(seconds, 1e-9))


def bell(n, centre, width):
    return np.exp(-(((times(n) - centre) / width) ** 2))


def distort(x, drive):
    return np.tanh(drive * x) / np.tanh(drive)


def saw_stack(phase, harmonics):
    out = np.zeros_like(phase)
    for k in range(1, harmonics + 1):
        out += np.sin(k * phase) / k ** 1.2
    return out


def place(buf, sig, seconds, wrap=False):
    """Adds sig into buf at a time. With wrap, the part past the end comes back to the start."""
    start = int(round(seconds * SR))
    if wrap:
        idx = (np.arange(len(sig)) + start) % len(buf)
        np.add.at(buf, idx, sig)
        return
    end = min(len(buf), start + len(sig))
    if end > start:
        buf[start:end] += sig[: end - start]


def edge_fade(x, seconds=0.004):
    """Short fade at both ends so one-shot sounds never click."""
    n = int(seconds * SR)
    if n > 0 and len(x) > 2 * n:
        ramp = np.linspace(0.0, 1.0, n)
        x[:n] *= ramp
        x[-n:] *= ramp[::-1]
    return x


def normalize(x):
    peak = np.max(np.abs(x))
    if peak <= 0.0:
        raise ValueError("silent sound")
    return x * (TARGET_PEAK / peak)


# ---------- gunshots, melee, UI ----------

def gunshot(seconds, rng, thump, thump_tau, crack_hp, crack_tau, body, body_tau, tail_hi, tail_tau,
            tail_gain, ring=None):
    """Crack (sharp top), thump (pitched-down bass), body (mid noise) and a room tail."""
    n = int(seconds * SR)
    x = filt(white(n, rng), lo=crack_hp) * decay(n, crack_tau)
    x += 0.9 * np.sin(glide(thump[0], thump[1], n)) * decay(n, thump_tau)
    x += body[0] * filt(white(n, rng), lo=body[1], hi=body[2]) * decay(n, body_tau)
    x += tail_gain * filt(pink(n, rng), hi=tail_hi) * decay(n, tail_tau)
    if ring is not None:
        x += ring[0] * tone(ring[1], n) * decay(n, ring[2])
    return edge_fade(x * attack(n, 0.0005))


def pistol_shot(variant):
    rng = rng_for(f"pistol_shot_{variant}")
    return gunshot(0.45, rng, thump=(120 + rng.uniform(-6, 6), 48), thump_tau=0.05,
                   crack_hp=3000, crack_tau=0.008, body=(0.55, 250, 2200), body_tau=0.08,
                   tail_hi=1100, tail_tau=0.18, tail_gain=0.22)


def shotgun_shot():
    rng = rng_for("shotgun_shot")
    return gunshot(0.7, rng, thump=(90, 35), thump_tau=0.09, crack_hp=1500, crack_tau=0.012,
                   body=(0.9, 200, 2500), body_tau=0.14, tail_hi=1000, tail_tau=0.35, tail_gain=0.32)


def rifle_shot():
    rng = rng_for("rifle_shot")
    return gunshot(1.1, rng, thump=(80, 40), thump_tau=0.06, crack_hp=2500, crack_tau=0.006,
                   body=(0.7, 300, 3000), body_tau=0.10, tail_hi=1400, tail_tau=0.42, tail_gain=0.42,
                   ring=(0.12, 180.0, 0.20))


def dry_fire():
    rng = rng_for("dry_fire")
    n = int(0.12 * SR)
    x = filt(white(n, rng), lo=2000) * decay(n, 0.004) + 0.45 * tone(1700, n) * decay(n, 0.008)
    return edge_fade(x)


def _metal_click(seconds, rng, freq, amp):
    n = int(seconds * SR)
    return amp * (filt(white(n, rng), lo=1500, hi=5000) * decay(n, 0.003)
                  + 0.4 * tone(freq, n) * decay(n, 0.02))


def reload():
    rng = rng_for("reload")
    x = np.zeros(int(1.0 * SR))
    place(x, _metal_click(0.2, rng, 2600, 0.8), 0.05)
    place(x, _metal_click(0.2, rng, 1900, 0.7), 0.42)
    place(x, _metal_click(0.2, rng, 3200, 1.0), 0.82)
    return edge_fade(x)


def swing_light():
    rng = rng_for("swing_light")
    n = int(0.42 * SR)
    x = np.zeros(n)
    # Four bands that peak one after another, so the air noise seems to sweep past
    for centre, peak in ((900, 0.10), (1400, 0.16), (2000, 0.22), (2800, 0.28)):
        x += filt(white(n, rng), lo=centre * 0.6, hi=centre * 1.5) * bell(n, peak, 0.08)
    return edge_fade(x)


def swing_heavy():
    rng = rng_for("swing_heavy")
    n = int(0.65 * SR)
    x = np.zeros(n)
    for centre, peak in ((300, 0.22), (500, 0.27), (800, 0.32)):
        x += filt(white(n, rng), lo=centre * 0.6, hi=centre * 1.5) * bell(n, peak, 0.11)
    x += 0.5 * np.sin(glide(160, 70, n)) * bell(n, 0.25, 0.12)
    return edge_fade(x)


def hit_flesh(variant):
    rng = rng_for(f"hit_flesh_{variant}")
    n = int(0.30 * SR)
    f0 = 130 + rng.uniform(-12, 12)
    x = np.sin(glide(f0, 45, n)) * decay(n, 0.06)
    x += 0.6 * filt(white(n, rng), hi=900) * decay(n, 0.03)
    x += 0.3 * filt(white(n, rng), lo=2000) * decay(n, 0.002)
    return edge_fade(x)


def hit_wood():
    rng = rng_for("hit_wood")
    n = int(0.25 * SR)
    x = 0.8 * tone(420, n) * decay(n, 0.03)
    x += 0.35 * tone(1190, n) * decay(n, 0.015)
    x += 0.5 * filt(white(n, rng), lo=1200, hi=3500) * decay(n, 0.008)
    return edge_fade(x)


def footstep_grass(variant):
    rng = rng_for(f"footstep_grass_{variant}")
    n = int(0.28 * SR)
    body = filt(white(n, rng), hi=1100) * decay(n, 0.05) * attack(n, 0.003)
    rustle = filt(white(n, rng), lo=3000, hi=6000) * decay(n, 0.02)
    return edge_fade(body + 0.12 * rustle)


def zombie_groan(variant):
    rng = rng_for(f"zombie_groan_{variant}")
    seconds = 1.6
    n = int(seconds * SR)
    t = times(n)
    base = (88, 102, 116)[variant - 1]
    # Pitch drops by about 28 % over the groan, with a slow 5 Hz wobble
    phase = glide(base, base * 0.72, n) + 0.03 * np.sin(2.0 * np.pi * 5.0 * t)
    voice = distort(filt(saw_stack(phase, 10), hi=900), 1.4)
    breath = filt(white(n, rng), lo=200, hi=700) * 0.25
    env = attack(n, 0.25) * np.minimum(1.0, (seconds - t) / 0.5) * (0.85 + 0.15 * np.sin(2 * np.pi * 4 * t))
    return edge_fade((voice + breath) * env, 0.01)


def zombie_attack():
    rng = rng_for("zombie_attack")
    seconds = 0.9
    n = int(seconds * SR)
    t = times(n)
    phase = glide(150, 95, n) + 0.05 * np.sin(2 * np.pi * 7 * t)
    snarl = distort(filt(saw_stack(phase, 12), hi=1800), 4.0)
    rasp = filt(white(n, rng), lo=1000, hi=2600) * (0.5 + 0.5 * np.sin(2 * np.pi * 28 * t)) * 0.4
    env = attack(n, 0.03) * np.minimum(1.0, (seconds - t) / 0.5)
    return edge_fade((snarl + rasp) * env)


def _dog_yelp(rng, seconds):
    n = int(seconds * SR)
    phase = glide(520, 340, n)
    voice = distort(filt(saw_stack(phase, 8), hi=1400), 2.5)
    grit = filt(white(n, rng), lo=600, hi=1800) * 0.3
    return (voice + grit) * attack(n, 0.01) * decay(n, seconds / 3.0)


def dog_bark():
    # Two short yelps with a gap: a bark, not a growl
    rng = rng_for("dog_bark")
    gap = np.zeros(int(0.14 * SR))
    x = np.concatenate([_dog_yelp(rng, 0.22), gap, _dog_yelp(rng, 0.2), np.zeros(int(0.1 * SR))])
    return edge_fade(x)


def pickup():
    x = np.zeros(int(0.3 * SR))
    place(x, np.sin(2 * np.pi * 1047 * times(int(0.2 * SR))) * decay(int(0.2 * SR), 0.06) * 0.8, 0.0)
    place(x, np.sin(2 * np.pi * 1568 * times(int(0.2 * SR))) * decay(int(0.2 * SR), 0.07) * 0.6, 0.07)
    return edge_fade(x)


def ui_click():
    rng = rng_for("ui_click")
    n = int(0.07 * SR)
    x = tone(1600, n) * decay(n, 0.012) + 0.3 * filt(white(n, rng), hi=4000) * decay(n, 0.003)
    return edge_fade(x)


def craft_done():
    x = np.zeros(int(1.1 * SR))
    for note, at in ((1046.5, 0.0), (1318.5, 0.12), (1568.0, 0.24)):
        m = int(0.85 * SR)
        t = times(m)
        # Bell-like: a quiet inharmonic partial that dies fast, under a longer main tone
        partial = np.sin(2 * np.pi * note * t) + 0.25 * np.sin(2 * np.pi * note * 2.76 * t) * decay(m, 0.12)
        place(x, partial * decay(m, 0.4) * 0.8, at)
    return edge_fade(x)


def level_up():
    x = np.zeros(int(1.2 * SR))
    notes = ((523.25, 0.0, 0.2), (783.99, 0.1, 0.2), (1046.5, 0.2, 0.2), (1318.5, 0.3, 0.7))
    for freq, at, tau in notes:
        m = int(1.0 * SR)
        t = times(m)
        tri = np.sin(2 * np.pi * freq * t) + 0.2 * np.sin(4 * np.pi * freq * t) + 0.05 * np.sin(6 * np.pi * freq * t)
        place(x, tri * attack(m, 0.005) * decay(m, tau) * 0.7, at)
    return edge_fade(x)


def player_hurt():
    rng = rng_for("player_hurt")
    n = int(0.45 * SR)
    t = times(n)
    phase = glide(210, 150, n) + 0.04 * np.sin(2 * np.pi * 6 * t)
    grunt = distort(filt(saw_stack(phase, 10), lo=250, hi=1300), 1.5) * decay(n, 0.15) * attack(n, 0.02)
    breath = filt(white(n, rng), lo=500, hi=1500) * decay(n, 0.1) * 0.2
    return edge_fade(grunt + breath)


def car_engine_loop():
    """One second of a 30 Hz engine idle. Built from whole cycles, so the loop joins without a click."""
    n = SR
    t = times(n)
    phase = 2 * np.pi * 30 * t
    rumble = saw_stack(phase, 8) * (0.75 + 0.25 * np.sin(phase))
    return filt(rumble, hi=700)


def car_horn():
    n = int(0.9 * SR)
    t = times(n)
    tone = np.sin(2 * np.pi * 420 * t) + np.sin(2 * np.pi * 520 * t)
    return edge_fade(distort(tone, 1.6) * attack(n, 0.01) * np.minimum(1.0, (0.9 - t) / 0.15))


def car_door():
    rng = rng_for("car_door")
    n = int(0.22 * SR)
    t = times(n)
    thunk = np.sin(2 * np.pi * 160 * t) * decay(n, 0.05)
    clatter = filt(white(n, rng), lo=600, hi=3000) * decay(n, 0.02) * 0.5
    return edge_fade(thunk + clatter)


def car_impact():
    rng = rng_for("car_impact")
    n = int(0.35 * SR)
    t = times(n)
    thud = np.sin(glide(110, 60, n)) * decay(n, 0.1)
    crunch = filt(white(n, rng), lo=1500, hi=6000) * decay(n, 0.05) * 0.6
    return edge_fade(thud + crunch)


# ---------- ambience loops (20 s each; built circularly so the end joins the start) ----------

def _gust(phases):
    """A slow swell that repeats exactly every 20 s (whole-number cycles only)."""
    t = times(int(LOOP_SECONDS * SR))
    gust = 0.6 + 0.25 * np.sin(2 * np.pi * 1 * t / LOOP_SECONDS + phases[0])
    gust += 0.15 * np.sin(2 * np.pi * 3 * t / LOOP_SECONDS + phases[1])
    return np.clip(gust, 0.05, 1.0)


def _bird_chirp(rng):
    notes = []
    for i in range(int(rng.integers(2, 5))):
        dur = rng.uniform(0.05, 0.09)
        f0 = rng.uniform(2600, 4200)
        f1 = f0 * rng.uniform(0.8, 1.25)
        m = int(dur * SR)
        s = np.sin(glide(f0, f1, m)) + 0.2 * np.sin(2 * glide(f0, f1, m))
        s = s * attack(m, 0.005) * decay(m, dur * 0.6)
        notes.append((s, i * (dur + rng.uniform(0.02, 0.05))))
    chirp = np.zeros(int((notes[-1][1] + 0.2) * SR))
    for s, at in notes:
        place(chirp, s, at)
    return chirp


def _cricket_burst(rng):
    pulse_len = int(0.025 * SR)
    spacing = 0.06
    freq = rng.uniform(3700, 4200)
    burst = np.zeros(int((4 * spacing + 0.05) * SR))
    for i in range(4):
        place(burst, np.sin(2 * np.pi * freq * times(pulse_len)) * np.hanning(pulse_len), i * spacing)
    return burst * 0.7


def day_loop():
    rng = rng_for("day_loop")
    n = int(LOOP_SECONDS * SR)
    wind = filt(pink(n, rng), lo=60, hi=900) * _gust((0.0, 1.3))
    whoosh = filt(brown(n, rng), lo=150, hi=500) * 0.25 * _gust((2.0, 0.4))
    x = wind + whoosh
    for _ in range(9):
        place(x, _bird_chirp(rng) * rng.uniform(0.12, 0.2), rng.uniform(0.0, LOOP_SECONDS), wrap=True)
    return x


def night_loop():
    rng = rng_for("night_loop")
    n = int(LOOP_SECONDS * SR)
    wind = filt(brown(n, rng), lo=40, hi=380) * 1.6 * _gust((0.7, 2.2))
    hush = filt(pink(n, rng), lo=400, hi=1200) * 0.06 * _gust((1.1, 0.2))
    x = wind + hush
    for period in (0.9, 1.1, 1.35):
        k = 0
        while k * period < LOOP_SECONDS:
            place(x, _cricket_burst(rng) * rng.uniform(0.05, 0.08), k * period + rng.uniform(0.0, 0.05), wrap=True)
            k += 1
    return x


# ---------- writing and reporting ----------

SFX = [
    ("pistol_shot_1", lambda: pistol_shot(1)), ("pistol_shot_2", lambda: pistol_shot(2)),
    ("shotgun_shot", shotgun_shot), ("rifle_shot", rifle_shot),
    ("dry_fire", dry_fire), ("reload", reload),
    ("swing_light", swing_light), ("swing_heavy", swing_heavy),
    ("hit_flesh_1", lambda: hit_flesh(1)), ("hit_flesh_2", lambda: hit_flesh(2)),
    ("hit_wood", hit_wood),
    ("footstep_grass_1", lambda: footstep_grass(1)), ("footstep_grass_2", lambda: footstep_grass(2)),
    ("footstep_grass_3", lambda: footstep_grass(3)), ("footstep_grass_4", lambda: footstep_grass(4)),
    ("zombie_groan_1", lambda: zombie_groan(1)), ("zombie_groan_2", lambda: zombie_groan(2)),
    ("zombie_groan_3", lambda: zombie_groan(3)),
    ("zombie_attack", zombie_attack), ("dog_bark", dog_bark),
    ("pickup", pickup), ("ui_click", ui_click), ("craft_done", craft_done),
    ("level_up", level_up), ("player_hurt", player_hurt),
    ("car_engine_loop", car_engine_loop), ("car_horn", car_horn),
    ("car_door", car_door), ("car_impact", car_impact),
]
AMBIENT = [("day_loop", day_loop), ("night_loop", night_loop)]


# ---------- wild animals (V12-02) ----------

def deer_snort():
    # A short breathy snort: airy noise over a low voiced buzz that falls in pitch
    rng = rng_for("deer_snort")
    seconds = 0.35
    n = int(seconds * SR)
    t = times(n)
    breath = filt(white(n, rng), lo=250, hi=1400) * 0.9
    voice = filt(saw_stack(glide(190, 110, n) + 0.04 * np.sin(2 * np.pi * 9 * t), 6), hi=900) * 0.5
    return edge_fade((breath + voice) * attack(n, 0.02) * decay(n, 0.09))


def rabbit_squeak():
    # A thin high squeak with a little wobble
    seconds = 0.25
    n = int(seconds * SR)
    t = times(n)
    phase = glide(1500, 950, n) + 0.06 * np.sin(2 * np.pi * 30 * t)
    x = np.sin(phase) + 0.3 * np.sin(2 * phase)
    return edge_fade(x * attack(n, 0.005) * decay(n, 0.07))


def _grunt(rng, seconds):
    n = int(seconds * SR)
    voice = distort(filt(saw_stack(glide(120, 80, n), 10), hi=700), 2.0)
    rasp = filt(white(n, rng), lo=200, hi=900) * 0.3
    return (voice + rasp) * attack(n, 0.03) * decay(n, seconds / 2.5)


def boar_grunt():
    # Two low grunts, one after the other
    rng = rng_for("boar_grunt")
    x = np.zeros(int(0.75 * SR))
    place(x, _grunt(rng, 0.3), 0.0)
    place(x, _grunt(rng, 0.35), 0.36)
    return edge_fade(x)


def boar_squeal():
    # A high, wavering scream that holds and then drops away
    rng = rng_for("boar_squeal")
    seconds = 0.9
    n = int(seconds * SR)
    t = times(n)
    phase = glide(1100, 700, n) + 0.05 * np.sin(2 * np.pi * 14 * t)
    voice = distort(filt(saw_stack(phase, 4), lo=300, hi=3000), 1.5)
    grit = filt(white(n, rng), lo=800, hi=2500) * 0.15
    env = attack(n, 0.02) * np.minimum(1.0, (seconds - t) / 0.25)
    return edge_fade((voice + grit) * env)


# The animal sounds are added after the functions above exist (SFX is read by main)
SFX += [
    ("deer_snort", deer_snort), ("rabbit_squeak", rabbit_squeak),
    ("boar_grunt", boar_grunt), ("boar_squeal", boar_squeal),
]


def write_wav(path, x):
    pcm = np.clip(np.round(x * 32767.0), -32768, 32767).astype("<i2")
    with wave.open(path, "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(SR)
        wav.writeframes(pcm.tobytes())


def peak_dbfs(x):
    pcm = np.clip(np.round(x * 32767.0), -32768, 32767)
    return 20.0 * np.log10(max(np.max(np.abs(pcm)), 1.0) / 32767.0)


def main():
    os.makedirs(SFX_DIR, exist_ok=True)
    os.makedirs(AMBIENT_DIR, exist_ok=True)
    for name, make in SFX:
        x = normalize(make())
        write_wav(os.path.join(SFX_DIR, name + ".wav"), x)
        print(f"sfx/{name:18s} {len(x) / SR:6.2f} s  peak {peak_dbfs(x):6.2f} dBFS")
    for name, make in AMBIENT:
        x = normalize(make())
        write_wav(os.path.join(AMBIENT_DIR, name + ".wav"), x)
        seam = abs(float(x[0]) - float(x[-1]))
        step = float(np.median(np.abs(np.diff(x))))
        print(f"ambient/{name:14s} {len(x) / SR:6.2f} s  peak {peak_dbfs(x):6.2f} dBFS"
              f"  loop seam step {seam:.5f} (median sample step {step:.5f})")


if __name__ == "__main__":
    main()
