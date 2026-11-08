#!/bin/sh
# Builds the Android host APK inside Docker (docs/platforms/MOBILE_ANDROID.md, A1).
# Two phases, as AGENTS.md allows ("--network=none after fetch"). Phase 1 runs with network: AGP resolves
# every artifact, including the detached configurations (for example aapt2) that `dependencies` does not
# resolve, and Gradle writes verification metadata. This means build plugins run with network in phase 1
# (recorded as an open follow-up). Phase 2 builds with --network none and --offline and verifies every
# artifact against that metadata.
# Inputs outside the repository, prepared by the owner's install (see odd/tasks/mobile-android.md):
#   AQELITH_LAB           default: /Volumes/Scheda SSD/aqelith-lab
#   $AQELITH_LAB/android-build-dl/gradle-9.1.0-bin.zip   Gradle distribution, SHA-256 checked below
#   $AQELITH_LAB/android-linux-sdk                       Linux build-tools 35.0.1 and 36.0.0, platform-tools, platform 35
#   $AQELITH_LAB/gradle-home                             Gradle cache (created here)
set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
LAB="${AQELITH_LAB:-/Volumes/Scheda SSD/aqelith-lab}"
GRADLE_ZIP="$LAB/android-build-dl/gradle-9.1.0-bin.zip"
LINUX_SDK="$LAB/android-linux-sdk"
GRADLE_HOME_CACHE="$LAB/gradle-home"
DEBUG_KEYSTORE_DIR="$LAB/android-debug-keystore"
# Pinned by digest: the JDK 17 image that was pulled and checked (odd/tasks/evidence/android-versions.txt).
IMAGE="eclipse-temurin:17-jdk@sha256:43338b737d0f75d6883d3c4563bc66974113ca8ac296e82f39d161fb3d398f0d"
GRADLE_SHA256=a17ddd85a26b6a7f5ddb71ff8b05fc5104c0202c6e64782429790c933686c806

test -f "$GRADLE_ZIP" || { echo "missing $GRADLE_ZIP" >&2; exit 2; }
# Verify the Gradle distribution before any container unpacks or runs it (SHA-256 published by Gradle).
test "$(shasum -a 256 "$GRADLE_ZIP" | cut -d' ' -f1)" = "$GRADLE_SHA256" || { echo "Gradle distribution SHA-256 mismatch: refusing to build" >&2; exit 2; }
test -d "$LINUX_SDK/build-tools/35.0.1" || { echo "missing Linux SDK at $LINUX_SDK" >&2; exit 2; }
mkdir -p "$GRADLE_HOME_CACHE" "$DEBUG_KEYSTORE_DIR"

# Runs one command in the pinned container. $1 is "network" or "none"; $2 is the command.
run_in_container() {
  net="$1"; cmd="$2"
  set -- docker run --rm --platform linux/amd64
  if [ "$net" = "none" ]; then set -- "$@" --network none; fi
  set -- "$@" \
    -v "$HERE:/work" \
    -v "$LINUX_SDK:/opt/android-sdk" \
    -v "$GRADLE_HOME_CACHE:/root/.gradle" \
    -v "$DEBUG_KEYSTORE_DIR:/root/.android" \
    -v "$GRADLE_ZIP:/opt/gradle-dist.zip:ro" \
    -w /work \
    -e ANDROID_HOME=/opt/android-sdk \
    -e ANDROID_SDK_ROOT=/opt/android-sdk \
    "$IMAGE" sh -c "$SETUP; $cmd"
  "$@"
}

# Unpacks Gradle into the persisted cache on first use (unzip is installed only when network is available).
SETUP='set -eu; G=/root/.gradle/dist/gradle-9.1.0; if [ ! -x $G/bin/gradle ]; then command -v unzip >/dev/null || (apt-get update -qq && apt-get install -y -qq unzip >/dev/null); mkdir -p /root/.gradle/dist; unzip -q /opt/gradle-dist.zip -d /root/.gradle/dist; fi; export PATH=$G/bin:$PATH'

echo "phase 1: fetch and write verification metadata (network on)"
run_in_container network "if [ ! -f gradlew ]; then gradle wrapper --gradle-version 9.1.0 --gradle-distribution-sha256-sum $GRADLE_SHA256; fi; ./gradlew --version >/dev/null; ./gradlew --no-daemon --write-verification-metadata sha256 :app:assembleDebug"

echo "phase 2: build (network none, offline, verified)"
run_in_container none "./gradlew --no-daemon --offline :app:assembleDebug"

APK="$HERE/app/build/outputs/apk/debug/app-debug.apk"
test -f "$APK" && echo "built: $APK" && shasum -a 256 "$APK"
