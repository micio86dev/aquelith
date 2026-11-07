# Mobile target: Android

Status: **A0 done: SDK and arm64 emulator installed on the external SSD and checked; the Android host (A1) is not started.** Evidence and open points: [odd/tasks/mobile-android.md](../../odd/tasks/mobile-android.md), log entries L4 to L7. First mobile target, by the owner's decision on 2026-11-07. Roadmap: [ROADMAP.md](ROADMAP.md). The second mobile target is iOS ([MOBILE_IOS.md](MOBILE_IOS.md)).

## Rules that apply (requirements; the host is planned, not built)

- Native UI, not a WebView wrapper ([PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 4). The rejected baseline is the Naze WebView wrapper ([TARGET_MATRIX.md](TARGET_MATRIX.md), row Android).
- Planned, not implemented: the same split as the desktop targets, where `aqelith view` exports the pages as JSON and a platform host draws them. The language would not know about Android ([DESKTOP_MACOS.md](DESKTOP_MACOS.md) describes the pattern for macOS).
- Native execution only. An emulator running on this machine is a test environment, not a claim of support for a device.

## Prerequisites (status)

1. A feature document, as the roadmap requires: [odd/tasks/mobile-android.md](../../odd/tasks/mobile-android.md). Written; owner approval recorded (log L2).
2. Owner authorization for the SDK download, recorded (log L4). Done on the external SSD: about 5.6 GiB installed, measured with `du -sh`. The sizes were measured before the install, from Google's public index (the feature document has the table).
3. Minimum Android version and host toolchain: not chosen yet; decided in A1.

## Milestones

Not defined yet. They will be written in the feature document.

## Limits, as they are today

- No Android code exists in this repository.
- Nothing here has been run on Android.
