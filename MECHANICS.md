# ROLLERBALL 3D — mechanics sheet

There is no original target in this workspace (clean-room rebuild from the
written brief only). Every value below is either taken from the brief or marked
**estimated**. The game was tuned to feel like a fast 1984-era video pinball
table: heavy ball, lively bumpers, strong flippers, and a four-screen drop.
The current presentation is a modern perspective-3D remake; the rules, physics
rate and scoring are unchanged from the playable 2D build.

## Display and layout

| Value | Number | Source |
|---|---|---|
| Virtual canvas | 320 × 240 (4:3) | estimated; MSX-inspired. The brief names the original 256 × 192; this 1.25× canvas keeps the same 4:3 grid and is displayed with integer nearest-neighbour scaling. |
| Window | 960 × 720 (3×) | estimated |
| Startup display | borderless fullscreen on a normal launch; `--windowed` or any capture run keeps the 960 × 720 window | request (person asked for fullscreen) |
| Fullscreen toggle | F11 or Alt+Enter switches fullscreen ↔ the centred 960 × 720 window | request |
| Fullscreen framing | the 4:3 canvas stays centred with dark bars; a 1920 × 1080 screen shows a 1440 × 1080 table at x = 240, never stretched | derived from the 4:3 layout |
| Fullscreen hint | "F11 FULLSCREEN" / "F11 WINDOWED" in the bottom of the sidebar | estimated |
| Start menu entries | 1 PLAYER, 2 PLAYERS, MATCH PLAY, EXIT GAME (four rows in the sidebar) | request (person asked for an exit option) |
| Exit game | select EXIT GAME and press Enter, or press Escape on the start menu | request |
| Exit key safety | the exit row ignores Space and the flipper start keys, so a table-control press cannot quit by accident | estimated |
| Capture runs | screenshot and self-test always stay windowed and unfocused | required for checks |
| Playfield share | 224 px = 70% | brief |
| HUD share | 96 px = 30% | brief |
| Screens | 4 stacked, 240 px each | brief |
| Table height | 960 px | derived |
| Pixel filtering | nearest neighbour | brief |
| HUD rows | PLAYER ONE, PLAYER TWO, HIGH SCORE, BONUS, BALLS | brief |

## 3D presentation

| Value | Number | Source |
|---|---|---|
| Presentation | one perspective-3D cabinet, tilted camera | request |
| Playfield art | rendered to a 448 x 1920 texture, nearest sampling, applied to the slab | estimated |
| Camera field of view | 42° vertical | estimated |
| Camera height | 315 table units above the playfield | estimated |
| Camera back offset | 250 table units behind the ball's screen | estimated |
| Camera look-ahead | 0 units (targets the current screen centre) | estimated |
| Follow smoothing | exponential 11 /s, same clamp 120…840 | estimated; existing 2D follow |
| Table slab | 238 x 974 x 12 units | estimated |
| Cabinet rail height | 17 units | estimated |
| Playfield surface | flat textured quad at z = 0.05 | estimated |
| Ball height above surface | 5 units (sphere radius 5) | derived from ball radius |
| Ball contact shadow | 19 x 19 soft radial shadow at z = 0.08 | estimated |
| Bumper hardware | barrel r=12.2 h=8, crown r=13 h=1.8, chrome cap, glowing dot | estimated |
| Flipper hardware | capsule radius 4.4, length 33.2, bright rubber top | estimated |
| Slot cabinet | raised frame around the printed reels, three torus rims | estimated |
| Drop targets | 18 x 6 x 7 raised bars, hide with the 2D target state | estimated |
| Spinner | 28 x 2.6 x 10 chrome paddle, inherits the 2D spin | estimated |
| Kickback holes | torus rings, red when empty, green when armed | estimated |
| Plunger | chrome shaft, red knob, four spring rings, 7-unit travel | estimated |
| Backdrop | 920 x 1500 starfield plane at z = -150 | estimated |
| Lighting | one 9 000 lux directional light, 320 cd/m² ambient, four coloured point lights (2 200 000 lm, range 260) | estimated |
| Bloom | 0.12 | estimated |
| UI compositing | fixed 960 x 720 opaque UI texture; only the right 672…960 sidebar slice is composited over the 3D table | estimated |
| UI texture size | fixed 960 x 720 regardless of window size, so fullscreen does not rebuild textures | derived |
| Camera viewports | table left 70% and sidebar 30% of the centred 4:3 rectangle | brief |
| Mode menu location | inside the sidebar (right 30%), never over the playfield | estimated |
| HUD viewport | left 70% table, right 30% sidebar | brief |

## Physics

| Value | Number | Source |
|---|---|---|
| Fixed physics rate | 120 Hz | brief |
| CCD | swept CCD against all body types + speculative CCD on ball | brief |
| Gravity | 1500 px/s² downward | estimated |
| Ball radius | 5 px | estimated |
| Ball restitution | 0.30 (combine: max) | estimated |
| Ball friction | 0.06 (combine: min) | estimated |
| Ball rotation | free rolling (not locked) | estimated fix: a locked ball could be held forever by static friction on the 24° flipper |
| Wall restitution | 0.45 | estimated |
| Wall friction | 0.08 | estimated |
| Resting / stuck watchdog | 6 s slow-ball nudge; 1 s lane relaunch; 1.1 s slot-cabinet sideways shove; 0.75 s flipper roll-off shove | estimated |
| Ball spawn | launch lane, x = 208.5, y = 30 | estimated |

## Flippers

| Value | Number | Source |
|---|---|---|
| Flipper length | 42 px physics (capsule) / 48 px art | estimated |
| Flipper width | 7 px physics / 10 px art | estimated |
| Pivot x | 62 px and 162 px | estimated |
| Pivot y | screen bottom + 38 px | estimated |
| Rest angle | ±24° from horizontal | estimated |
| Raised angle | ∓24° (48° total sweep) | brief: limited rotation |
| Raise top speed | 30 rad/s, accelerated at 1500 rad/s² | estimated |
| Return top speed | 13 rad/s, accelerated at 320 rad/s² | estimated |
| End-stop braking | 2400 rad/s², exact clamp at 0° and 48° | estimated |
| Full stroke time | about 0.033 s up / 0.075 s back | derived |
| Motion clock | fixed 120 Hz physics delta (frame-rate independent) | derived |
| Contact impulse | up to 1500 px/s along the working face; strength grows with distance from the pivot (about 300 px/s at the hinge, 900–1500 px/s at the tip) | estimated |
| Held / stationary flipper | passive surface: no launch; a held flipper skips the rescue, otherwise a slow ball rolls or is nudged off | estimated |
| Kick cooldown | 0.06 s per flipper | estimated |
| Rubber | restitution 0.25, friction 0.35 (combine max) | estimated fix: 0.9 exceeded tan(24°) and let a still ball hang on the face |
| Flipper roll-off rescue | after 0.75 s below 55 px/s on a non-driven flipper: 220 px/s downhill + 55 px/s off the face | estimated fix for the reported hang |
| Funnel end gap | 14 px outlane between wall end and pivot | estimated |
| Active flippers | only the pair on the current camera screen | brief |

## Plunger

| Value | Number | Source |
|---|---|---|
| Charge time to full | 0.95 s | estimated |
| Launch speed | 1760 + 620 × charge px/s | estimated |
| Minimum / maximum speed | 1760 / 2380 px/s | estimated |
| Lane exit trigger | y ≥ 880 px | estimated |
| Exit hand-off | placed at (188, 908) with velocity (−310, 150) px/s | estimated |

The launch lane is one-way: the ball is handed into the top screen after the
shot instead of falling back into the chute.

## Camera

| Value | Number | Source |
|---|---|---|
| View height | 240 px | derived |
| Follow smoothing | exponential, 11 /s | estimated |
| Vertical clamp | 120 … 840 px | derived |
| At top/bottom | camera snaps to the clamp so a fast ball is never off-screen | estimated |

## Scoring

| Event | Points | Other effect | Source |
|---|---|---|---|
| Bumper | 100 | bounce kick | estimated |
| Slingshot | 50 | strong kick | estimated |
| A/B/C rollover / arrow | 250 | +100 bonus | estimated |
| Complete A+B+C | 2 500 | +1 000 bonus, +1 multiplier; arms kickbacks | estimated |
| Drop target | 500 | +100 bonus | estimated |
| All three drop targets | 5 000 | +1 000 bonus, +1 multiplier, reset after 1.6 s | estimated |
| Spinner entry | 100 | spins the spinner | estimated |
| Slot start | 200 | starts reels | estimated |
| Kickback hole (armed) | 1 000 | strong save; disarms | estimated |
| Kickback hole (empty) | 100 | weak bounce | estimated |
| Chute | 100 | chute sound | estimated |
| Bonus start | 1 000 | resets every ball | estimated |
| Bonus multiplier | ×1 start | increased by ABC/Slot/Drop wins | brief |
| End-of-ball bonus | BONUS × multiplier, 20 ticks × 0.075 s after 0.35 s delay | added to score audibly and visibly | brief |
| Balls per player | 3 | extra balls add one | brief |
| Match Play start | 5 000 each | your points are subtracted from the opponent | brief (optional mode) |

## Slot machine (screen 2)

| Value | Number | Source |
|---|---|---|
| Symbols | bell, cherries, aubergine | brief |
| Spin time / stops | 0.85 s, 1.20 s, 1.55 s | estimated |
| Reel refresh | 0.075 s | estimated |
| Cooldown after spin | 2.5 s | estimated |
| Three bells | +3 000 and an extra ball | estimated |
| Three cherries | +2 000, +1 multiplier, +1 000 bonus | estimated |
| Three aubergines | +5 000 | estimated |
| Any pair | +500 | estimated |
| No match | +100 | estimated |
| Cabinet footprint | 88 × 38 px solid box at (112, 620) | estimated |
| Cabinet cap | 12 px high peaked roof, 44 px slopes at about 15°, 3 px thick | estimated fix |
| Slot-area escape | if the ball stays under 40 px/s near the cabinet for 1.1 s, add 300 px/s sideways and 260 px/s up | estimated fix |

## Nudge / tilt

| Value | Number | Source |
|---|---|---|
| Nudge impulse | ±210 px/s horizontal, +330 px/s up | estimated |
| Nudge limit | 3 within a 3 s window | brief: short cooldown / tilt limit |
| Tilt | 3 s with flippers disabled | estimated |

## Controls

| Action | Keys | Source |
|---|---|---|
| Left flipper | Left Shift, Z, Left Arrow | brief |
| Right flipper | Right Shift, M, Right Arrow | brief |
| Plunger | hold and release Space | brief |
| Nudge | W or Up Arrow | brief |
| Start / confirm | Enter, Space, Z, M, Left/Right Shift, Left/Right Arrow or left mouse click | estimated |
| Quick start | 1 = one player, 2 = two players, 3 = Match Play | estimated |
| Menu navigation | Up/Down or W/S, wraps through the four entries | estimated |
| Exit game | select EXIT GAME + Enter, or Escape at the start menu | request |
| Pause | P | estimated |
| Music on/off | N | estimated addition |
| Screenshot | F12 writes shots/screenshot.png | addition |
| Back to menu | Escape | estimated |

## Audio

| Value | Setting | Source |
|---|---|---|
| Backend | bevy_kira_audio / Kira; overlapping voices are mixed by Kira | brief |
| Files | `assets/audio/*.wav` and `.ogg`, mapped by `sounds.ron` | brief |
| Shipped clips | 19 files (18 WAV + 1 OGG) covering 15 events | `assets/audio/sounds.ron` |
| Style of shipped clips | original AY-3-8910-style recreations: 3 square channels, noise, 16-step envelopes, sweeps, arpeggios; not original recordings | estimated; `tools/generate_audio.py` |
| Variation | bumper 3 takes, flipper 2, rollover lane 2; one chosen at random per hit | estimated |
| Loading | every file is decoded before playback; a missing, empty, corrupt or wrong-format file uses the synth instead of a silent handle | `src/audio.rs` |
| Fallback | built-in square/triangle/noise PSG patches for all 15 events | brief |
| Event voices | max 16 queued per frame; per-effect trims of −7 to −14 dB under a −6 dB channel | estimated |
| Polyphony guard | dedicated SFX channel with a 3:1 compressor at −14 dB (4 ms attack, 140 ms release) | estimated |
| Events | Start, PlungerCharge, PlungerRelease, FlipperHit, BumperHit, RolloverLane, SlotSpin, SlotWin, ChuteDrop, BonusCountdownTick, BallDrain, GameOver, Kickback, ExtraBall, Tilt | brief + estimated additions |
| Start jingle | 1.25 s PSG fanfare when a game begins | estimated |
| Tilt | 0.75 s buzzer (the old build wrongly used the game-over sting) | estimated |
| Kickback | 0.40 s thump/whoosh when an armed kickback hole fires | estimated |
| Extra ball | 0.95 s rising fanfare when three bells award a ball | estimated |

## Background music

| Value | Number | Source |
|---|---|---|
| Track | `assets/audio/music_theme.wav`, mapped by `music` in `sounds.ron` | request |
| Style | original early-80s 8-bit pinball chiptune, two squares + triangle bass + noise drums | request |
| Tempo / length | 126 BPM, 8 bars, 15.24 s seamless loop | estimated; file measured |
| Key / progression | A minor, Am–F–C–G | estimated |
| Channel | dedicated Kira channel at −16 dB, separate from the SFX compressor | estimated |
| Polyphony | loops continuously; effects mix on top | request |
| Toggle | N mutes/unmutes without touching the effects | estimated addition |
| Fallback | if the file is missing/corrupt, `synth_music()` builds a procedural 8-bit loop at start-up | estimated |
| Format support | `.wav` and `.ogg` via the same decoder as the effects | brief |
| Ownership | composed for this remake; no 1984 recording or sample is shipped | clean-room |
