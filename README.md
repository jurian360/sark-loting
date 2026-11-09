# SARK Loting

Offline Windows-programma om startnummers te loten voor de deelnemers van een rally.

## Downloaden

GitHub → **Actions** → laatste run van *Build Windows* → artifact **SARK-Loting-windows**:

- `SARK-Loting-Setup.exe` – installer (startmenu + bureaublad-snelkoppeling)
- `SARK-Loting-portable.exe` – zonder installatie te starten

Een tag `v*` pushen (bijv. `v1.0.0`) maakt ook een GitHub Release met beide bestanden.

## Gebruik

1. **CSV openen** – kolommen: `EQ nummers, Navigator - Achternaam, Navigator - Voornaam, Bestuurder - Achternaam, Bestuurder - Voornaam`
   (komma of puntkomma; al ingevulde EQ-nummers worden als vast nummer overgenomen).
2. Vul de **nummers in de loting** in, bijv. `1-57` of `1-40, 45, 50-60`.
3. Vul eventueel bij **Vast nr.** een vooraf bepaald nummer in. Die nummers gaan uit de loting en die deelnemers worden overgeslagen.
4. **Start loting** – in de volgorde van de CSV: klik **Draai** (rad) of vul een nummer in bij **Handmatig nummer** → **Toewijzen**.
   **Ongedaan maken** zet de laatste trekking terug.
5. **Exporteren** – slaat de CSV op met de EQ-nummers ingevuld (zelfde kolommen en volgorde).
