# ROLLERBALL 3D

![De vier schermen tellende tafel in perspectief, met de vaste retro-zijbalk rechts](docs/screenshot.png)

Een speelbare flipperkast met vier schermen, gemaakt in Rust met Bevy 0.20 en
Avian2D 0.8. Het is een clean-room eerbetoon aan de MSX-flipperklassieker
*Rollerball* uit 1984, opnieuw opgebouwd in een moderne 3D-stijl: vier
gestapelde schermen, flippers op elk scherm, A/B/C-banen, bumpers, slingshots,
drop targets, kickback-gaten, een gokkast met drie rollen, een bonusaftelling,
drie ballen, 1P/2P en een Match Play-scoreduel.

De regels en de physics op 120 Hz zijn dezelfde als in de eerdere 2D-versie.
De presentatie is nu een gekantelde 3D-kast: een bedrukt speelveld ligt op een
plaat, met verhoogde randen, chromen palen, oplichtende bumpers, een kast met
drie rollen, een plunger en een gepolijste stalen bal erboven. De camera volgt
de bal soepel omhoog en omlaag over de vier schermen. De zijbalk rechts is nog
steeds een vaste, bewust retro HUD van 30% van de breedte.

Er is ook een originele, continu lopende 8-bit flipperkastmuziek op een eigen
audiokanaal, met de toets N als demper.

Het modusmenu staat in de vaste zijbalk in plaats van over de tafel, zodat de
kast vanaf het eerste beeld zichtbaar en bespeelbaar is. Enter, Spatie, een
flipperknop of een muisklik start een spel voor één speler; 2 en 3 springen
direct naar twee spelers of Match Play. Met de pijltjes (of W/S) verplaats je
de markering, en de vierde keuze, EXIT GAME, sluit het spel met Enter — handig
in volledig scherm, waar geen sluitknop van het venster is. Escape doet op het
startmenu hetzelfde. Zodra een spel begint, maken de moduslabels plaats voor de
live scores, de naam van het scherm en de meldingen. F12 slaat een schermafbeelding op.

De achtergrond van het speelveld is het aangeleverde ruimtebeeld
(`assets/backgrounds/space_background.png`): de planeet staat op het onderste
scherm en de oplichtende sporen van de bal lopen omhoog over het bovenste
scherm. Een breedbeeld foto kan niet zonder vervorming op een tafel van
224 x 960 worden uitgerekt, dus hij wordt in vier beeldverhoudinggetrouwe
kolommen geknipt, één per scherm, met een donkere waas eroverheen zodat de
inserts en scorepanelen leesbaar blijven.

## Downloaden

De broncode staat op GitHub:
<https://github.com/simondewaal1980/rollerball-3d>

### Kant-en-klaar spelen (Linux, geen Rust nodig)

De [nieuwste release](https://github.com/simondewaal1980/rollerball-3d/releases/latest)
bevat een kant-en-klare Linux x86_64-versie. Pak de map uit, laat `assets` naast
het programma staan en start `./rollerball`. `LEESMIJ.txt` in de download (en
`docs/RUN-LINUX.txt` in de broncode) bevat dezelfde stappen in het Nederlands,
met de volledige toetsenlijst.

### Zelf bouwen

```sh
git clone https://github.com/simondewaal1980/rollerball-3d
cd rollerball-3d
cargo run --release
```

Je hebt een recente stabiele Rust-toolchain nodig (`rustup` installeert die).
Op Debian of Ubuntu komen de venster- en audiobibliotheken uit:

```sh
sudo apt install build-essential pkg-config libasound2-dev libudev-dev \
  libx11-dev libxi-dev libxcursor-dev libxrandr-dev libwayland-dev \
  libxkbcommon-dev
```

De eerste build duurt een paar minuten, omdat Bevy groot is. Het voltooide spel
start direct vanuit `target/release/rollerball`. Het is gebouwd en getest op
Linux (x86_64), waar geluid via ALSA loopt.

## Starten

```sh
cargo run --release
```

Het spel opent in randloos volledig scherm op de huidige monitor. Het complete
flipperdoek van 320 x 240 blijft op zijn 4:3-verhouding en wordt gecentreerd met
donkere balken, zodat er op een breedbeeldscherm niets wordt uitgerekt. Met F11
(of Alt+Enter) schakel je op elk moment terug naar het venster van 960 x 720;
dezelfde toetsen gaan weer naar volledig scherm. Onderaan de zijbalk staat
altijd de actuele toets. Begin met

```sh
cargo run --release -- --windowed
```

om vanaf het eerste beeld het kleine venster te houden.

Voor een snelle automatische controle:

```sh
cargo run -- --screenshot shots/table.png
cargo run -- --self-test selftest.json
```

De schermafbeeldingoptie opent het venster zonder de aandacht op te eisen en
sluit af nadat de afbeelding is opgeslagen. De zelftest bekijkt het startmenu
(inclusief de keuze EXIT GAME), drukt op dezelfde starttoets als een speler,
laadt en lost de plunger, flipt, meet de bal, de flipper en de HUD, controleert
dat het modusmenu uit de zijbalk verdwijnt, laat de bal boven de gokkast vallen
om te bevestigen dat hij vrij doorrolt, en legt een bal op de afgeronde rand van
een flipper om te bevestigen dat hij eraf rolt in plaats van blijft hangen.
Daarna schrijft hij het JSON-resultaat en sluit af.

## Besturing

| Actie | Toetsen |
|---|---|
| Linkerflipper | Linker Shift, Z, pijltje links |
| Rechterflipper | Rechter Shift, M, pijltje rechts |
| Plunger | Spatie ingedrukt houden, loslaten om te schieten |
| Nudge / tiltwaarschuwing | W of pijltje omhoog |
| Starten / bevestigen | Enter, Spatie, Z/M, linker/rechter Shift, pijltje links/rechts of een muisklik |
| Snel starten | 1 één speler, 2 twee spelers, 3 Match Play |
| Pauze | P |
| Volledig scherm / venster | F11 of Alt+Enter |
| Muziek aan/uit | N |
| Schermafbeelding | F12 (slaat `shots/screenshot.png` op) |
| Terug naar menu | Escape |
| Spel afsluiten | EXIT GAME in het startmenu, dan Enter; Escape in het startmenu sluit ook af |

Alleen de flippers op het scherm waar de bal zich bevindt, werken.

## Geluid

Het spel zoekt `assets/audio/sounds.ron`, dat elk geluidsgebeurtenis koppelt aan
een `.wav`- of `.ogg`-bestand en een volume in dB. Zet je eigen bestanden in
`assets/audio/` en wijs het RON-bestand ernaar; opnieuw compileren is niet
nodig. Een gebeurtenis mag meerdere bestanden noemen, en het spel kiest bij
elke treffer willekeurig één ervan — de drie bumpers en twee flippervarianten
zorgen dat herhaalde treffers niet identiek klinken.

Alle vijftien gebeurtenissen hebben een geluid (achttien WAV-bestanden en
`slot_win.ogg`). Het zijn originele nabootsingen in de stem van de MSX
AY-3-8910 PSG, geen samples uit het spel van 1984: drie blokgolfkanalen, ruis,
volumenveloppen met 16 stappen, toonhoogteglijders en korte arpeggio's. De
bestanden worden vóór het afspelen gedecodeerd; een ontbrekend, leeg of beschadigd
bestand wordt vervangen door een procedurele PSG-klank die in de code wordt
gemaakt, zodat het spel altijd geluid heeft. Bij het opstarten meldt het logboek
hoeveel bestanden zijn geladen en precies welke gebeurtenissen de terugval gebruiken.
Heb je een opname van de originele tafel, geef de bestanden dan deze namen (of
pas `sounds.ron` aan); het spel gebruikt ze bij de volgende start.

De achtergrondmuziek is `assets/audio/music_theme.wav`, gekoppeld via de
`music`-vermelding in `sounds.ron`. Het is een originele 8-matige chiptune in
a-klein (Am-F-C-G) op 126 BPM met twee blokgolfstemmen, een driehoeksbas en een
ruisdrumkit, geen opname uit 1984. De muziek loopt in een lus op een eigen
Kira-kanaal op -16 dB en kan worden vervangen door elk `.wav`- of `.ogg`-bestand
van 15 seconden of langer. Ontbreekt het bestand of is het onleesbaar, dan maakt
`synth_music()` een procedurele 8-bit-lus in de code. Met N demp je de muziek
zonder de effecten te raken.

`tools/generate_audio.py` bouwt de meegeleverde effectenbank opnieuw op, en
`tools/generate_music.py` doet hetzelfde voor de soundtrack.

## Opmerkingen bij de 3D-presentatie

De kunst van het speelveld wordt gerenderd naar een textuur van 448 x 1920 met
nearest-neighbour-filtering en op de 3D-plaat gelegd, zodat elk origineel
baanlabel, lampje en scorepaneel leesbaar blijft terwijl de perspectiefcamera
diepte geeft. De hardware erboven bestaat uit echte meshes: cilinders en
toruskronen voor de bumpers, capsules voor de flippers, een kastframe met drie
rollen, kickback-ringen die van rood naar groen kleuren als ze scherp staan, een
chromen plunger, drop-targetstaven en een UV-bol met een metaalmateriaal voor de
bal. Het aangeleverde ruimtebeeld vult de achtergrond van het speelveld, terwijl
een zwak sterrenveld en cyaan/magenta puntlichten voorkomen dat de kast in een
leegte zweeft. De kunstlaag gebruikt een verlengd verre vlak (2000 in plaats van
1000), zodat decals achter z=0 — achtergrond, schermnaden en panelen — niet uit
de bedrukte textuur worden weggeknipt. De kast wordt altijd samengesteld binnen
een gecentreerd 4:3-gebied. In volledig scherm krijgt een breedbeeldmonitor
donkere zijbalken in plaats van een uitgerekte tafel, en de zijbalk houdt zijn
exacte breedte van 30%. Alle camerawaarden en presentatiegetallen staan in
`MECHANICS.md` en `mechanics.json`.

## Opmerkingen bij de physics

De simulatie gebruikt een vaste stap van 120 Hz. De bal heeft swept CCD tegen
elk lichaamstype plus speculative CCD, zodat een plungerschot van 2000+ px/s
niet door een flipper kan schieten. Flippers zijn kinematische capsules met een
beperkte zwaai van 48°. Hun beweging loopt op de vaste physicsklok (en is dus
niet afhankelijk van de framerate), ze remmen exact op beide eindstanden, en een
treffer wordt geschaald naar de plek waar de bal het rubber raakt: zwak bij het
scharnier, het sterkst aan de punt. Een bal die op een opgehouden flipper rust,
wordt niet de lucht in geschoten. De bal rolt nu vrij in plaats van met
vastgezette rotatie, zodat hij van een passief rubbervlak afrolt in plaats van
eraan te blijven hangen; een langzame bal die na 0,75 s nog steeds op het rubber,
de afgeronde punt of het scharnier zit, krijgt een zachte duw bergafwaarts. De
trechterwanden stoppen vóór de scharnieren, zodat er een echte uitloop ontstaat
in plaats van een zak die de bal vasthoudt. De gokkast heeft een puntdak in
plaats van een vlakke plank, zodat een landende bal eraf rolt; blijft er toch een
bal aarzelen, dan geeft een reddingsduwtje na 1,1 s hem een zetje opzij. Alle
afstelbare getallen staan in `MECHANICS.md` en `mechanics.json`.
