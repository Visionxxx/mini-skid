```
 _____ _____ _____ _____    _____ _____ _____ ____
|     |     |   | |     |  |   __|  |  |     |    \
| | | |-   -| | | |-   -|  |__   |    -|-   -|  |  |
|_|_|_|_____|_|___|_____|  |_____|__|__|_____|____/
```

# Mini Skid

**A top-down 2D racing game inspired by the legendary [Super Skidmarks](https://www.lemonamiga.com/games/details.php?id=1788) (1995, Amiga)**

> *Built with Rust + [macroquad](https://macroquad.rs). Runs native and in the browser via WebAssembly.*

---

## Play Now

**[>>> PLAY IN BROWSER <<<](https://visionxxx.github.io/mini-skid/)**

---

## Screenshots

*Oval Speedway / Figur-8 Cross / Fjord Circuit*

## Features

- **6 biler** — 2 spillere (WASD + piltaster) + 4 AI med personlighet
- **3 baner** — Oval Speedway, Figur-8 med bro, Fjord Circuit med hopp
- **Terreng** — bakker, hopp-ramper, bro/overgang, helningseffekter
- **Drift-fysikk** — skidmerker, røykskyer, gnister
- **Screen shake** — ved kollisjoner og landinger
- **Detaljer** — frontlykter, baklys, eksosflammer, tilskuere, flagg
- **WASM** — spilles direkte i nettleseren, under 700 KB totalt

## Controls

| Spiller 1 | Spiller 2 | Funksjon |
|-----------|-----------|----------|
| `W` | `Up` | Gass |
| `S` | `Down` | Brems / revers |
| `A` | `Left` | Sving venstre |
| `D` | `Right` | Sving hoyre |

| Tast | Funksjon |
|------|----------|
| `P` | Pause |
| `R` | Restart race |
| `ESC` | Tilbake til meny |

## Build

```bash
# Native
cargo run --release

# Web (WASM)
./build-web.sh
cd web && python3 -m http.server 8080
```

Krever: `rustup target add wasm32-unknown-unknown`

## Tech

| | |
|---|---|
| Spraak | Rust |
| Grafikk | macroquad 0.4 (miniquad) |
| Rendering | Primitiver — ingen teksturer eller sprites |
| WASM-storrelse | ~650 KB |
| Avhengigheter | 0 runtime, 0 node_modules |

---

## Tribute

This game is a humble homage to **Super Skidmarks** (1995) by
**Acid Software** from New Zealand — one of the greatest top-down
racers ever made for the Amiga.

```
CREDITS — SUPER SKIDMARKS (1995)
─────────────────────────────────
Code ........... Andrew Blackbourn
Graphics ....... Hans Butler
                 Kurt Butler
                 Rodney Smith
Music .......... Anthony Milas
Support ........ Simon Armstrong
Publisher ...... Acid Software
─────────────────────────────────
Platforms: Amiga ECS/AGA, CD32,
           Sega Mega Drive
Players:   1—4 simultaneous
Reviews:   92% Amiga Format
           89% Amiga Computing
```

Super Skidmarks proved that you didn't need polygons or
Mode 7 to make a racing game that was pure, chaotic fun.
24 tracks. 8 car classes from Minis to Formula 1.
4-player mayhem with a multitap.

Andrew Blackbourn wrote it in **68k assembler and Blitz Basic**
on an Amiga. In 1995.

If you haven't played the original — find an emulator and do it.

---

*Made with Rust, caffeine, and Claude Code.*
