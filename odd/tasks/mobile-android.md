# Feature: mobile target on Android, milestone A0 (decision and measurement)

Objective: start the first mobile target, Android, with the prerequisites that the roadmap requires before any code: an approved feature document, a measured toolchain footprint, and the owner's OK for each large download. Roadmap: [docs/platforms/ROADMAP.md](../../docs/platforms/ROADMAP.md). Target page: [docs/platforms/MOBILE_ANDROID.md](../../docs/platforms/MOBILE_ANDROID.md).
Branch: docs/mobile-android-a0-done (from main); the feature branch docs/mobile-android-feature was merged earlier. Delivery strategy: ask-on-risk (default). Test runner: none for this document; `gga run` before each commit.
No SDK, emulator, or system image is downloaded by this feature. Installing any of them needs the owner's explicit OK.

## Specs

S1. The first mobile target is Android; iOS is the second. Owner, 2026-11-07, verbatim: "iOS come secondo target mobile, metti prima android".
S2. Native UI, not a WebView wrapper ([docs/platforms/PLATFORM_STRATEGY.md](../../docs/platforms/PLATFORM_STRATEGY.md), section 4). The rejected baseline is the Naze WebView wrapper.
S3. Same split as the desktop targets: `aqelith view` exports the pages as JSON and a platform host draws them. The language does not know about Android.
S4. Native execution only. An emulator on this Mac is a test environment, not a claim of support for a device.
S5. No paid services. No large installs without the owner's explicit OK. Internal disk free space is a measured value, not a fixed one: 4.7 GiB at task A0.2, 4.3 GiB before the cache cleanup, 5.7 GiB after it, 5.2 GiB later (see the Log, L5). The external SSD had about 38 GiB free at A0.2 (measured 2026-11-07).
S6. Every statement in the Android documents has evidence: a command output, a measured size, or a URL with its access date.
S7. Assumption (not yet approved): the host is written in Kotlin with the platform's native UI toolkit; the minimum Android version is not chosen. Both are decided in milestone A1, not here.

## Tasks

| ID | Linked | Route | Status | Commit or evidence |
|---|---|---|---|---|
| A0.1 | S1, S5, S6 | inline | done | this document and the measured table above |
| A0.2 | S5 | inline | done | `df -h /` on 2026-11-07: internal 4.7 GiB free; `df -h` on the external SSD: 38 GiB free. Later readings are in L5 |
| A0.3 | S4, S5 | inline | done (2026-11-07, owner authorized) | SDK on the external SSD at /Volumes/Scheda SSD/android-sdk: platform-tools, platforms;android-35, build-tools;35.0.1, emulator 37.3.8.0 (build 16847572, macOS arm64), system-images;android-35;google_apis;arm64-v8a. Installed size 5.6 GiB (`du -sh`). Emulator checked with `emulator -version`. |
| A1 | S2, S3, S7 | delegated, decided in A1 | not started | needs A0.3 and a written A1 feature section |

### Evidence: measured footprint (2026-11-07, measured from Google's public package index before the install; no package was downloaded for the measurement)

Source: `https://dl.google.com/android/repository/repository2-3.xml` (packages) and `https://dl.google.com/android/repository/sys-img/google_apis/sys-img2-3.xml` (system images). Sizes are the archive sizes for macOS hosts, in MiB. The installed size on disk is larger than these archive sizes; it was measured later, at task A0.3 (5.6 GiB, `du -sh`).

| Component | Size (MiB) | Needed for |
|---|---:|---|
| cmdline-tools;latest | 148.4 | the `sdkmanager` tool |
| platform-tools | 15.8 | `adb` |
| platforms;android-35 | 61.3 | compile against API 35 |
| build-tools;35.0.1 | 73.3 | build and package |
| emulator | 469.3 | run an emulator |
| system-images;android-35;google_apis;arm64-v8a | 1696.5 | an emulator image for Apple silicon |
| system-images;android-35;google_apis;x86_64 | 1658.3 | an emulator image for Intel (not needed on this Mac) |

Minimal set without a system image: 768.1 MiB (sum of the first five rows). With the arm64 API 35 image: about 2.4 GiB. At task A0.2 the internal disk had 4.7 GiB free, so the minimal set (768.1 MiB) fits there; the arm64 image (about 2.4 GiB in total) would leave little margin. The external SSD (about 38 GiB free) was used for the margin.

## Log

L1. 2026-11-07. Owner, verbatim: "iOS come secondo target mobile, metti prima android".
L2. 2026-11-07. Owner asked for the next step; I proposed the Android feature document and a size measurement without downloading. Owner: "Si procedi".
L3. Measured the SDK footprint from the public index (table above). The measured minimal set is 768.1 MiB; the system image dominates. These are archive sizes, not installed sizes.
L4. 2026-11-07. Owner authorized the install ("Autorizzo tutto"). Done on the external SSD. sdkmanager first installed the macOS x64 emulator build; that build runs under Rosetta on Apple silicon, so it was not accepted. The arm64 build (macosx aarch64, same Google index, SHA-1 bb3f4feea6428632fe5bd0b98473d37a62cbbe77, checked with `shasum -a 1 -c`; SHA-1 is weak, so this check only matches Google's published index and is not a signature check) was installed instead and is the one in use. The x64 copy was then deleted, after the arm64 build was verified (log L8).
L5. 2026-11-07. Owner authorization for this cleanup, verbatim: "Fai spazio sul disco primario se necessario, guarda Docker, cache... Basta che lo fai in totale sicurezza". Primary disk readings (`df -h /`, 2026-11-07): 4.7 GiB at A0.2; 4.3 GiB just before the cleanup; 5.7 GiB after the cleanup; 5.2 GiB at the end of A0 (the SDK downloads ran on the primary disk for the temporary files, before they were moved). The cleanup freed space by pruning regenerable caches only (Docker build cache and unused untagged images, Homebrew cleanup, npm, pnpm store prune, uv cache). Free space went from 4.3 GiB to 5.7 GiB. Docker volumes, containers, tagged images, Chrome, Playwright, Gradle, cargo, and Claude caches were not touched.
L6. Evidence: `emulator -version` prints `Android emulator version 37.3.8.0 (build_id 16847572)`. `file` output: `emulator/emulator: Mach-O 64-bit executable arm64` and `emulator/qemu-img: Mach-O 64-bit executable arm64`. The system image's `build.prop` contains `ro.product.cpu.abi=arm64-v8a`.
L7. Next: A1, the Android host. Java on this machine: `java -version` prints `openjdk version "26.0.1" 2026-04-21` (command output saved to the external SSD, file java-version.txt). The Android Gradle Plugin may need JDK 17 or 21 instead; that is checked in A1 before any build. Gradle caches should go to GRADLE_USER_HOME on the external SSD, not the primary disk.
L8. 2026-11-07. Cleanup of our own download artifacts: the x64 emulator copy and the extracted download folders were deleted from the external SSD after the arm64 build was verified. These were files this task created, not owner data, and can be re-downloaded from the same index. The Android SDK folder itself is kept. No separate approval line was recorded for this step; the owner's authorization of the install and of cleanup (L4, L5) is the basis.
