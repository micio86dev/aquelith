# Mobile target: Android

Status: **proposed, not started; the feature document is written and measured (A0).** First mobile target, by the owner's decision on 2026-11-07. Roadmap: [ROADMAP.md](ROADMAP.md). The second mobile target is iOS ([MOBILE_IOS.md](MOBILE_IOS.md)).

## Rules that apply

- Native UI, not a WebView wrapper ([PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 4). The rejected baseline is the Naze WebView wrapper ([TARGET_MATRIX.md](TARGET_MATRIX.md), row Android).
- The same split as the desktop targets: `aqelith view` exports the pages as JSON, and a platform host draws them. The language does not know about Android ([DESKTOP_MACOS.md](DESKTOP_MACOS.md) describes the pattern).
- Native execution only. An emulator running on this machine is a test environment, not a claim of support for a device.

## What is needed before any work starts

1. A feature document, written and approved, as the roadmap requires: [odd/tasks/mobile-android.md](../../odd/tasks/mobile-android.md) (status: awaiting owner approval of the measured footprint).
2. The owner's approval to download the Android SDK, the build tools, and an emulator system image. Their sizes have not been measured yet. The internal disk has about 4.7 GiB free; the external SSD has about 38 GiB free, and the SDK could be placed there.
3. A decision on the minimum Android version and the host toolchain. Neither is chosen yet.

## Milestones

Not defined yet. They will be written in the feature document.

## Limits, as they are today

- No Android code exists in this repository.
- Nothing here has been run on Android.
