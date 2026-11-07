# Feature: mobile target on Android, milestone A0 (decision and measurement)

Objective: start the first mobile target, Android, with the prerequisites that the roadmap requires before any code: an approved feature document, a measured toolchain footprint, and the owner's OK for each large download. Roadmap: [docs/platforms/ROADMAP.md](../../docs/platforms/ROADMAP.md). Target page: [docs/platforms/MOBILE_ANDROID.md](../../docs/platforms/MOBILE_ANDROID.md).
Branch: docs/mobile-android-feature (from main). Delivery strategy: ask-on-risk (default). Test runner: none for this document; `gga run` before each commit.
No SDK, emulator, or system image is downloaded by this feature. Installing any of them needs the owner's explicit OK.

## Specs

S1. The first mobile target is Android; iOS is the second. Owner, 2026-11-07, verbatim: "iOS come secondo target mobile, metti prima android".
S2. Native UI, not a WebView wrapper ([docs/platforms/PLATFORM_STRATEGY.md](../../docs/platforms/PLATFORM_STRATEGY.md), section 4). The rejected baseline is the Naze WebView wrapper.
S3. Same split as the desktop targets: `aqelith view` exports the pages as JSON and a platform host draws them. The language does not know about Android.
S4. Native execution only. An emulator on this Mac is a test environment, not a claim of support for a device.
S5. No paid services. No large installs without the owner's explicit OK. The internal disk has about 4.7 GiB free; the external SSD has about 38 GiB free (measured 2026-11-07).
S6. Every statement in the Android documents has evidence: a command output, a measured size, or a URL with its access date.
S7. Assumption (not yet approved): the host is written in Kotlin with the platform's native UI toolkit; the minimum Android version is not chosen. Both are decided in milestone A1, not here.

## Measured footprint (2026-11-07, from Google's public package index, no SDK downloaded)

Source: `https://dl.google.com/android/repository/repository2-3.xml` (packages) and `https://dl.google.com/android/repository/sys-img/google_apis/sys-img2-3.xml` (system images). Sizes are the archive sizes for macOS hosts, in MiB. The installed size on disk is larger and was not measured.

| Component | Size (MiB) | Needed for |
|---|---:|---|
| cmdline-tools;latest | 148.4 | the `sdkmanager` tool |
| platform-tools | 15.8 | `adb` |
| platforms;android-35 | 61.3 | compile against API 35 |
| build-tools;35.0.1 | 73.3 | build and package |
| emulator | 469.3 | run an emulator |
| system-images;android-35;google_apis;arm64-v8a | 1696.5 | an emulator image for Apple silicon |
| system-images;android-35;google_apis;x86_64 | 1658.3 | an emulator image for Intel (not needed on this Mac) |

Minimal set without a system image: 768.1 MiB (sum of the first five rows). With the arm64 API 35 image: about 2.4 GiB. The external SSD can hold this; the internal disk cannot.

## Tasks

| ID | Linked | Route | Status | Commit or evidence |
|---|---|---|---|---|
| A0.1 | S1, S5, S6 | inline | done | this document and the measured table above |
| A0.2 | S5 | inline | done | `df -h` on 2026-11-07: internal 4.7 GiB free, external 38 GiB free |
| A0.3 | S4, S5 | inline | pending owner | install the SDK on the external SSD only after the owner's OK |
| A1 | S2, S3, S7 | delegated, decided in A1 | not started | needs A0.3 and a written A1 feature section |

## Log

L1. Owner, verbatim: "iOS come secondo target mobile, metti prima android".
L2. Owner asked for the next step; I proposed the Android feature document and a size measurement without downloading. Owner: "Si procedi".
L3. Measured the SDK footprint from the public index (table above). The measured minimal set is 768.1 MiB; the system image dominates. These are archive sizes, not installed sizes.
L4. Next step: the owner decides whether to install the SDK on the external SSD. Nothing is installed.
