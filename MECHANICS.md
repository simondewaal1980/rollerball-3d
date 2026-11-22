# ROLLERBALL 3D — speelwaarden

Er is geen origineel doel in deze werkmap (een clean-room rebuild op basis van
alleen de geschreven opdracht). Elke waarde hieronder komt ofwel uit de opdracht
of is als **geschat** gemarkeerd. Het spel is afgesteld om te voelen als een
snelle videoflipperkast uit 1984: een zware bal, levendige bumpers, krachtige
flippers en een val over vier schermen. De huidige presentatie is een moderne
3D-remake in perspectief; de regels, de physics-frequentie en de puntentelling
zijn ongewijzigd ten opzichte van de speelbare 2D-versie.

## Beeld en indeling

| Waarde | Getal | Bron |
|---|---|---|
| Virtueel doek | 320 × 240 (4:3) | geschat; MSX-geïnspireerd. De opdracht noemt de originele 256 × 192; dit doek van 1,25× houdt hetzelfde 4:3-raster en wordt met gehele nearest-neighbour-schaling weergegeven. |
| Venster | 960 × 720 (3×) | geschat |
| Beeld bij het starten | randloos volledig scherm bij een normale start; `--windowed` of een opname-run houdt het venster van 960 × 720 | opdracht (persoon vroeg om volledig scherm) |
| Wisselen volledig scherm | F11 of Alt+Enter wisselt tussen volledig scherm en het gecentreerde venster van 960 × 720 | opdracht |
| Kader in volledig scherm | het 4:3-doek blijft gecentreerd met donkere balken; een scherm van 1920 × 1080 toont een tafel van 1440 × 1080 op x = 240, nooit uitgerekt | afgeleid van de 4:3-indeling |
| Hint volledig scherm | "F11 FULLSCREEN" / "F11 WINDOWED" onderaan de zijbalk | geschat |
| Keuzes startmenu | 1 PLAYER, 2 PLAYERS, MATCH PLAY, EXIT GAME (vier regels in de zijbalk) | opdracht (persoon vroeg om een afsluitoptie) |
| Spel afsluiten | kies EXIT GAME en druk op Enter, of druk op Escape in het startmenu | opdracht |
| Veiligheid afsluitkeuze | de afsluitregel negeert Spatie en de flipper-starttoetsen, zodat een tafeltoets het spel niet per ongeluk kan sluiten | geschat |
| Opname-runs | schermafbeelding en zelftest blijven altijd in een venster zonder focus | vereist voor de controles |
| Aandeel speelveld | 224 px = 70% | opdracht |
| Aandeel HUD | 96 px = 30% | opdracht |
| Schermen | 4 gestapeld, elk 240 px | opdracht |
| Hoogte tafel | 960 px | afgeleid |
| Pixelfiltering | nearest neighbour | opdracht |
| HUD-regels | PLAYER ONE, PLAYER TWO, HIGH SCORE, BONUS, BALLS | opdracht |

## 3D-presentatie

| Waarde | Getal | Bron |
|---|---|---|
| Presentatie | één kast in perspectief, gekantelde camera | opdracht |
| Kunst speelveld | gerenderd naar een textuur van 448 x 1920, nearest sampling, op de plaat gelegd | geschat |
| Beeldhoek camera | 42° verticaal | geschat |
| Hoogte camera | 315 tafeleenheden boven het speelveld | geschat |
| Afstand camera naar achteren | 250 tafeleenheden achter het scherm van de bal | geschat |
| Vooruitkijken camera | 0 eenheden (richt op het midden van het huidige scherm) | geschat |
| Volgdeming | exponentieel 11 /s, zelfde begrenzing 120…840 | geschat; bestaande 2D-volging |
| Tafelplaat | 238 x 974 x 12 eenheden | geschat |
| Hoogte kastrand | 17 eenheden | geschat |
| Oppervlak speelveld | platte textuurquad op z = 0,05 | geschat |
| Hoogte bal boven oppervlak | 5 eenheden (bolstraal 5) | afgeleid van de balstraal |
| Contactschaduw bal | zachte radiale schaduw van 19 x 19 op z = 0,08 | geschat |
| Bumperhardware | cilinder r=12,2 h=8, kroon r=13 h=1,8, chromen kap, oplichtende stip | geschat |
| Flipperhardware | capsuleradius 4,4, lengte 33,2, helder rubber bovenop | geschat |
| Gokkast | verhoogd frame rond de bedrukte rollen, drie torusranden | geschat |
| Drop targets | verhoogde staven van 18 x 6 x 7, verborgen met de 2D-doelstatus | geschat |
| Spinner | chromen peddel van 28 x 2,6 x 10, erft de 2D-draai | geschat |
| Kickback-gaten | torusringen, rood als ze leeg zijn, groen als ze scherp staan | geschat |
| Plunger | chromen schacht, rode knop, vier veerringen, slag van 7 eenheden | geschat |
| Achtergrond | sterrenveldvlak van 920 x 1500 op z = -150 | geschat |
| Belichting | één richtlicht van 9 000 lux, omgevingslicht van 320 cd/m², vier gekleurde puntlichten (2 200 000 lm, bereik 260) | geschat |
| Bloom | 0,12 | geschat |
| UI-samenstelling | vaste ondoorzichtige UI-textuur van 960 x 720; alleen de zijbalkstrook 672…960 wordt over de 3D-tafel samengesteld | geschat |
| Grootte UI-textuur | vast 960 x 720 ongeacht de venstergrootte, zodat volledig scherm geen texturen opnieuw opbouwt | afgeleid |
| Camerapanelen | tafel links 70% en zijbalk 30% van het gecentreerde 4:3-rechthoek | opdracht |
| Plaats modusmenu | in de zijbalk (rechter 30%), nooit over het speelveld | geschat |
| HUD-paneel | links 70% tafel, rechts 30% zijbalk | opdracht |

## Physics

| Waarde | Getal | Bron |
|---|---|---|
| Vaste physics-frequentie | 120 Hz | opdracht |
| CCD | swept CCD tegen alle lichaamstypes + speculative CCD op de bal | opdracht |
| Zwaartekracht | 1500 px/s² omlaag | geschat |
| Balstraal | 5 px | geschat |
| Restitutie bal | 0,30 (combine: max) | geschat |
| Wrijving bal | 0,06 (combine: min) | geschat |
| Rotatie bal | vrij rollend (niet vastgezet) | geschatte oplossing: een vastgezette bal kon door statische wrijving op de flipper van 24° eeuwig worden vastgehouden |
| Restitutie muur | 0,45 | geschat |
| Wrijving muur | 0,08 | geschat |
| Rust- / vastloopbewaking | duwtje voor langzame bal na 6 s; herstart in de baan na 1 s; zetje opzij bij de gokkast na 1,1 s; afrolduwtje van de flipper na 0,75 s | geschat |
| Startpositie bal | lanceerbaan, x = 208,5, y = 30 | geschat |

## Flippers

| Waarde | Getal | Bron |
|---|---|---|
| Flipperlengte | 42 px physics (capsule) / 48 px kunst | geschat |
| Flipperbreedte | 7 px physics / 10 px kunst | geschat |
| Draaipunt x | 62 px en 162 px | geschat |
| Draaipunt y | onderkant scherm + 38 px | geschat |
| Rusthoek | ±24° ten opzichte van horizontaal | geschat |
| Opgeheven hoek | ∓24° (totale zwaai 48°) | opdracht: beperkte rotatie |
| Topsnelheid omhoog | 30 rad/s, versneld met 1500 rad/s² | geschat |
| Topsnelheid terug | 13 rad/s, versneld met 320 rad/s² | geschat |
| Remmen op eindstand | 2400 rad/s², exact begrensd op 0° en 48° | geschat |
| Duur volledige slag | ongeveer 0,033 s omhoog / 0,075 s terug | afgeleid |
| Bewegingsklok | vaste physicsdelta van 120 Hz (onafhankelijk van framerate) | afgeleid |
| Contactimpuls | tot 1500 px/s langs het werkende vlak; sterkte groeit met de afstand tot het draaipunt (ongeveer 300 px/s bij het scharnier, 900–1500 px/s aan de punt) | geschat |
| Vastgehouden / stilstaande flipper | passief oppervlak: geen lancering; een vastgehouden flipper slaat de redding over, anders rolt of duwt een langzame bal eraf | geschat |
| Wachttijd tussen trappen | 0,06 s per flipper | geschat |
| Rubber | restitutie 0,25, wrijving 0,35 (combine max) | geschatte oplossing: 0,9 was hoger dan tan(24°) en liet een stilstaande bal aan het vlak hangen |
| Afrolredding flipper | na 0,75 s onder 55 px/s op een niet-aangedreven flipper: 220 px/s bergafwaarts + 55 px/s van het vlak af | geschatte oplossing voor het gemelde blijven hangen |
| Opening aan einde trechter | uitloop van 14 px tussen einde muur en draaipunt | geschat |
| Actieve flippers | alleen het paar op het scherm waar de camera staat | opdracht |

## Plunger

| Waarde | Getal | Bron |
|---|---|---|
| Oplaadtijd tot vol | 0,95 s | geschat |
| Lanceersnelheid | 1760 + 620 × lading px/s | geschat |
| Minimum- / maximumsnelheid | 1760 / 2380 px/s | geschat |
| Trigger baanuitgang | y ≥ 880 px | geschat |
| Overdracht bij uitgang | geplaatst op (188, 908) met snelheid (−310, 150) px/s | geschat |

De lanceerbaan is eenrichtingsverkeer: de bal wordt na het schot in het
bovenste scherm geplaatst in plaats van terug te vallen in de goot.

## Camera

| Waarde | Getal | Bron |
|---|---|---|
| Hoogte beeld | 240 px | afgeleid |
| Volgdeming | exponentieel, 11 /s | geschat |
| Verticale begrenzing | 120 … 840 px | afgeleid |
| Boven / onder | de camera springt naar de begrenzing, zodat een snelle bal nooit buiten beeld raakt | geschat |

## Puntentelling

| Gebeurtenis | Punten | Ander effect | Bron |
|---|---|---|---|
| Bumper | 100 | stuiterzetje | geschat |
| Slingshot | 50 | krachtige trap | geschat |
| A/B/C-rollover / pijl | 250 | +100 bonus | geschat |
| A+B+C voltooid | 2 500 | +1 000 bonus, +1 vermenigvuldiger; kickbacks scherp | geschat |
| Drop target | 500 | +100 bonus | geschat |
| Alle drie drop targets | 5 000 | +1 000 bonus, +1 vermenigvuldiger, na 1,6 s reset | geschat |
| Ingang spinner | 100 | laat de spinner draaien | geschat |
| Start gokkast | 200 | start de rollen | geschat |
| Kickback-gat (scherp) | 1 000 | krachtige redding; ontwapent | geschat |
| Kickback-gat (leeg) | 100 | zwakke stuiter | geschat |
| Goot | 100 | gootgeluid | geschat |
| Start bonus | 1 000 | reset bij elke bal | geschat |
| Bonusvermenigvuldiger | ×1 bij de start | verhoogd door winst op ABC / gokkast / drop targets | opdracht |
| Bonus aan einde bal | BONUS × vermenigvuldiger, 20 tikken × 0,075 s na 0,35 s wachttijd | hoorbaar en zichtbaar bij de score opgeteld | opdracht |
| Ballen per speler | 3 | extra ballen voegen er één toe | opdracht |
| Start Match Play | 5 000 elk | jouw punten worden bij de tegenstander afgetrokken | opdracht (optionele modus) |

## Gokkast (scherm 2)

| Waarde | Getal | Bron |
|---|---|---|
| Symbolen | bel, kersen, aubergine | opdracht |
| Draaitijd / stops | 0,85 s, 1,20 s, 1,55 s | geschat |
| Verversing rollen | 0,075 s | geschat |
| Wachttijd na draai | 2,5 s | geschat |
| Drie bellen | +3 000 en een extra bal | geschat |
| Drie kersen | +2 000, +1 vermenigvuldiger, +1 000 bonus | geschat |
| Drie aubergines | +5 000 | geschat |
| Elk paar | +500 | geschat |
| Geen match | +100 | geschat |
| Voetafdruk kast | massief vak van 88 × 38 px op (112, 620) | geschat |
| Dak van de kast | puntdak van 12 px hoog, 44 px hellingen van ongeveer 15°, 3 px dik | geschatte oplossing |
| Ontsnapping gokkastgebied | blijft de bal 1,1 s onder 40 px/s bij de kast, voeg dan 300 px/s opzij en 260 px/s omhoog toe | geschatte oplossing |

## Nudge / tilt

| Waarde | Getal | Bron |
|---|---|---|
| Nudge-impuls | ±210 px/s horizontaal, +330 px/s omhoog | geschat |
| Nudgelimiet | 3 binnen een venster van 3 s | opdracht: korte wachttijd / tiltlimiet |
| Tilt | 3 s met uitgeschakelde flippers | geschat |

## Besturing

| Actie | Toetsen | Bron |
|---|---|---|
| Linkerflipper | Linker Shift, Z, pijltje links | opdracht |
| Rechterflipper | Rechter Shift, M, pijltje rechts | opdracht |
| Plunger | Spatie ingedrukt houden en loslaten | opdracht |
| Nudge | W of pijltje omhoog | opdracht |
| Starten / bevestigen | Enter, Spatie, Z, M, linker/rechter Shift, pijltje links/rechts of klik met de linkermuisknop | geschat |
| Snel starten | 1 = één speler, 2 = twee spelers, 3 = Match Play | geschat |
| Menunavigatie | Omhoog/Omlaag of W/S, loopt door de vier keuzes | geschat |
| Spel afsluiten | kies EXIT GAME + Enter, of Escape in het startmenu | opdracht |
| Pauze | P | geschat |
| Muziek aan/uit | N | geschatte toevoeging |
| Schermafbeelding | F12 schrijft shots/screenshot.png | toevoeging |
| Terug naar menu | Escape | geschat |

## Geluid

| Waarde | Instelling | Bron |
|---|---|---|
| Backend | bevy_kira_audio / Kira; overlappende stemmen worden door Kira gemengd | opdracht |
| Bestanden | `assets/audio/*.wav` en `.ogg`, gekoppeld via `sounds.ron` | opdracht |
| Meegeleverde clips | 19 bestanden (18 WAV + 1 OGG) voor 15 gebeurtenissen | `assets/audio/sounds.ron` |
| Stijl meegeleverde clips | originele nabootsingen in AY-3-8910-stijl: 3 blokgolfkanalen, ruis, enveloppen met 16 stappen, glijders, arpeggio's; geen originele opnamen | geschat; `tools/generate_audio.py` |
| Variatie | bumper 3 varianten, flipper 2, rollover-baan 2; per treffer willekeurig één gekozen | geschat |
| Laden | elk bestand wordt vóór het afspelen gedecodeerd; een ontbrekend, leeg, beschadigd of verkeerd bestand gebruikt de synth in plaats van een stille handle | `src/audio.rs` |
| Terugval | ingebouwde blokgolf-/driehoek-/ruis-PSG-klanken voor alle 15 gebeurtenissen | opdracht |
| Stemmen per gebeurtenis | max. 16 per frame in de wachtrij; trims per effect van −7 tot −14 dB onder een kanaal van −6 dB | geschat |
| Polyfoniebewaking | eigen SFX-kanaal met een compressor van 3:1 op −14 dB (4 ms attack, 140 ms release) | geschat |
| Gebeurtenissen | Start, PlungerCharge, PlungerRelease, FlipperHit, BumperHit, RolloverLane, SlotSpin, SlotWin, ChuteDrop, BonusCountdownTick, BallDrain, GameOver, Kickback, ExtraBall, Tilt | opdracht + geschatte toevoegingen |
| Startjingle | PSG-fanfare van 1,25 s wanneer een spel begint | geschat |
| Tilt | zoemer van 0,75 s (de oude build gebruikte ten onrechte de game-over-toon) | geschat |
| Kickback | dreun/whoosh van 0,40 s wanneer een scherp kickback-gat afgaat | geschat |
| Extra bal | stijgende fanfare van 0,95 s wanneer drie bellen een bal geven | geschat |

## Achtergrondmuziek

| Waarde | Getal | Bron |
|---|---|---|
| Nummer | `assets/audio/music_theme.wav`, gekoppeld via `music` in `sounds.ron` | opdracht |
| Stijl | originele 8-bit flipperkastchiptune uit het vroege 8-bit-tijdperk, twee blokgolven + driehoeksbas + ruisdrums | opdracht |
| Tempo / lengte | 126 BPM, 8 maten, naadloze lus van 15,24 s | geschat; bestand gemeten |
| Toonsoort / progressie | a-klein, Am–F–C–G | geschat |
| Kanaal | eigen Kira-kanaal op −16 dB, apart van de SFX-compressor | geschat |
| Polyfonie | loopt continu; effecten mengen eroverheen | opdracht |
| Schakelaar | N dempt/ontdempt zonder de effecten te raken | geschatte toevoeging |
| Terugval | ontbreekt of is het bestand beschadigd, dan bouwt `synth_music()` bij het starten een procedurele 8-bit-lus | geschat |
| Formaatondersteuning | `.wav` en `.ogg` via dezelfde decoder als de effecten | opdracht |
| Eigendom | gecomponeerd voor deze remake; geen opname of sample uit 1984 wordt meegeleverd | clean-room |

*De machineleesbare versie van deze waarden staat in `mechanics.json`. Daarin
blijven de veldnamen Engels, omdat de controlefuncties van de werkmap ze zo
lezen; de betekenis is dezelfde als in deze Nederlandse tabel.*
