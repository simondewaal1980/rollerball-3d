# Rollerball 3D 1.0 — Linux

Een kant-en-klare versie van de vier-schermen flipperkast in Rust. Je hoeft
niets te compileren.

## Zo start je

1. Download `rollerball-3d-1.0.0-linux-x86_64.tar.gz` hieronder.
2. Pak het bestand uit.
3. Start `rollerball` in de uitgepakte map (dubbelklikken of `./rollerball`).
   De map `assets` moet naast het programma blijven staan.
4. Werkt dubbelklikken niet? Voer eerst `chmod +x rollerball` uit.

## Wat zit erin

- Vier aaneengesloten schermen met flippers, bumpers, slingshots,
  rollover-banen, drop targets, kickback-gaten en een gokkast met drie rollen.
- Vaste 120 Hz physics met continue botsingsdetectie, zodat de bal niet door
  muren of flippers schiet.
- 1 speler, 2 spelers en Match Play, bonusaftelling, drie ballen en een
  hoogste score die bewaard blijft.
- Modulaire audio: eigen WAV/OGG-bestanden in `assets/audio/` met een
  `sounds.ron`-koppeling, plus een procedurele PSG-fallback en een originele
  8-bit soundtrack.
- Volledig scherm met F11 of Alt+Enter, en een EXIT GAME-item in het menu.

## Toetsen

| Actie | Toetsen |
|---|---|
| Linkerflipper | Linker Shift, Z, pijltje links |
| Rechterflipper | Rechter Shift, M, pijltje rechts |
| Plunger | Spatie ingedrukt houden, loslaten |
| Nudge | W of pijltje omhoog |
| Starten | Enter, Spatie of een flipperknop |
| Snel starten | 1, 2 of 3 |
| Pauze | P |
| Volledig scherm | F11 of Alt+Enter |
| Muziek aan/uit | N |
| Terug naar menu | Escape |
| Stoppen | EXIT GAME in het menu, dan Enter |

## Let op

Deze remake is een moderne driedimensionale interpretatie van de beschreven
tafel. De originele 1984-opnamen en het originele artwork zijn niet gebruikt:
de klanken, de soundtrack en de vormgeving zijn opnieuw gemaakt. De layout,
scores en het flippergevoel zijn zo dicht mogelijk bij de beschrijving gebracht
en staan met hun herkomst in `MECHANICS.md`.

Getest op Ubuntu 26.04 met een Intel-grafische kaart. Het spel gebruikt ALSA
voor geluid en de standaard Vulkan/OpenGL-stuurprogramma's voor beeld.
