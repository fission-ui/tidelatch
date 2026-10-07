# TideLatch

TideLatch is a one-touch ocean momentum game built with Fission. Hold the latch
to pull your craft toward the nearest anchor, release to sling across the water,
collect drifting cargo, and return it to harbour before a collision costs you
the haul.

This repository is both a playable game prototype and a compact example of
Fission's retained 2D scenes, deterministic game runtime, portable input, and
optional Rapier physics.

## Targets

- `linux`
- `macos`
- `windows`
- `web`
- `ios`
- `android`

## Play

- Hold **Space** to latch onto the nearest anchor; release it to preserve your momentum.
- On touch or pointer-only devices, use **Latch** and **Release**.
- Collect gold cargo crates and return to the harbour at the lower-left.
- Banking several crates together awards a larger score.
- Striking rocks drops one unbanked crate.

## Commands

- `fission run --target macos --project-dir .` -- launch the macOS prototype
- `fission doctor --project-dir .` -- check local SDKs, browsers, emulators, and Rust targets
- `fission devices --project-dir .` -- list runnable desktop, browser, simulator, emulator, and device targets
- `fission run --project-dir .` -- launch the desktop app and attach to output
- `fission run --target web --project-dir .` -- launch the web app and attach to the local server
- `fission run --target ios --project-dir .` -- build, install, launch, and attach to simulator logs
- `fission run --target android --project-dir .` -- build, install, launch, and attach to Android logs
- `fission run --target <target> --device <id> --detach --project-dir .` -- launch without attaching
- `fission logs --target <target> --device <id> --project-dir . --follow` -- attach later where supported
- `fission build --target <target> --project-dir . --release` -- build a target without launching it
- `fission test --target <target> --project-dir .` -- run the generated platform smoke test
- `fission add-target web ios android --project-dir .` -- scaffold more targets
- `fission add-capability storage --project-dir .` -- enable SQLite storage and add only the target-specific dependencies and Web assets it needs
- `fission add-capability filesystem nfc notifications biometric passkeys bluetooth barcode-scanner camera geolocation haptics microphone volume-control wifi --project-dir .` -- declare host capabilities and update platform config where possible
- `cat platforms/<target>/README.md` -- inspect target-specific prerequisites and environment variables

## Assets

- `assets/app-icon.png` is the default app icon seed copied from Fission's `docs/fission_logo.png`

## Status

The first playable prototype shares one game and scene implementation across all
six graphical targets. macOS is the first interactive qualification target;
the remaining generated shells are included for follow-up platform validation.
