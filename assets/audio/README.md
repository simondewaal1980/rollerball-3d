# Rollerball-geluid

Het spel zoekt in deze map de bestanden die in `sounds.ron` staan.
Toegestane formaten zijn `.wav` en `.ogg`. Vervang een genoemd bestand door je
eigen opname, of pas `sounds.ron` aan en wijs naar een nieuw bestand.

De negentien meegeleverde clips zijn originele nabootsingen in de stem van de
MSX AY-3-8910 PSG — drie blokgolfkanalen, ruis, volumenveloppen met 16 stappen
en korte arpeggio's — geen samples uit het spel van 1984. Ze dekken alle
vijftien geluidsgebeurtenissen, en sommige gebeurtenissen hebben meer dan één
variant:

    flipper_hit.wav, flipper_hit_2.wav        (twee varianten)
    bumper_hit.wav, _2.wav, _3.wav            (drie varianten)
    rollover_lane.wav, rollover_lane_2.wav    (twee varianten)

Een gebeurtenis met meerdere varianten wordt willekeurig afgespeeld, zodat een
reeks bumpers niet mechanisch hetzelfde klinkt. `slot_win` wordt bewust als
`.ogg` meegeleverd om te laten zien dat beide formaten werken.

Heb je een opname van de originele tafel, zet de bestanden dan onder deze namen
in deze map (of pas `sounds.ron` aan); ze worden bij de volgende start gebruikt
en je hoeft niets opnieuw te compileren. Bij het opstarten meldt het spel het
aantal, bijvoorbeeld:

    Audio: 19 external sound file(s) ready, 0 event(s) using the procedural synth

Ontbreekt een bestand, is het leeg, beschadigd of heeft het de verkeerde indeling,
dan blijft die gebeurtenis werken: het spel maakt er zelf een korte vierkante/
ruisige PSG-klank voor. Het opstartlogboek noemt elke gebeurtenis die de
terugval nodig heeft.

`volume_db` is een correctie in decibel per effect. De waarden zijn bewust
bescheiden, zodat meerdere bumpers tegelijk kunnen klinken zonder te clippen.

`../tools/generate_audio.py` bouwt de meegeleverde clips opnieuw op vanuit code.
