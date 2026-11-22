# Rollerball audio

The game looks in this folder for the files listed in `sounds.ron`.
Accepted formats are `.wav` and `.ogg`. Either replace a listed filename with
your own recording, or edit `sounds.ron` to point at a new file.

The nineteen shipped clips are original recreations in the voice of the MSX
AY-3-8910 PSG — three square-wave channels, noise, 16-step volume envelopes and
short arpeggios — not samples from the 1984 game. They cover all fifteen sound
events, and some events list more than one take:

    flipper_hit.wav, flipper_hit_2.wav        (two takes)
    bumper_hit.wav, _2.wav, _3.wav            (three takes)
    rollover_lane.wav, rollover_lane_2.wav    (two takes)

An event with several takes is played at random, so a string of bumper hits does
not sound mechanically repeated. `slot_win` deliberately ships as `.ogg` to show
both formats working.

If you own a recording of the original table, drop the files in here under these
names (or edit `sounds.ron`) and they are used on the next start; no rebuild is
needed. Start-up reports the count, for example:

    Audio: 19 external sound file(s) ready, 0 event(s) using the procedural synth

If a file is missing, empty, corrupt or has the wrong format, that event keeps
working: the game synthesizes a short square/noise PSG-style sound for it
instead. The start-up log names any event that needs the fallback.

`volume_db` values are per-effect decibel trims. They are deliberately modest
so several bumpers can sound at once without clipping.

`../tools/generate_audio.py` rebuilds the shipped clips from code.
