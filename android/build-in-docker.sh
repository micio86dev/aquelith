#!/bin/sh
# Builds the Android host APK inside Docker with JDK 17 (docs/platforms/MOBILE_ANDROID.md, A1).
# Two phases, as AGENTS.md requires for untrusted code: phase 1 fetches dependencies with network;
# phase 2 builds with --network none and --offline, so nothing is fetched during the build.
# Inputs outside the repository, prepared by the owner's install (see odd/tasks/mobile-android.md):
#   AQELITH_LAB           default: /Volumes/Scheda SSD/aqelith-lab
#   $AQELITH_LAB/android-build-dl/gradle-9.1.0-bin.zip   Gradle distribution, SHA-256 checked
#   $AQELITH_LAB/android-linux-sdk                       Linux build-tools 35.0.1, platform-tools, platform 35
#   $AQELITH_LAB/gradle-home                             Gradle cache (created here)
set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
LAB="${AQELITH_LAB:-/Volumes/Scheda SSD/aqelith-lab}"
GRADLE_ZIP="$LAB/android-build-dl/gradle-9.1.0-bin.zip"
LINUX_SDK="$LAB/android-linux-sdk"
GRADLE_HOME_CACHE="$LAB/gradle-home"
DEBUG_KEYSTORE_DIR="$LAB/android-debug-keystore"
IMAGE="eclipse-temurin:17-jdk"

test -f "$GRADLE_ZIP" || { echo "missing $GRADLE_ZIP" >&2; exit 2; }
test -d "$LINUX_SDK/build-tools/35.0.1" || { echo "missing Linux SDK at $LINUX_SDK" >&2; exit 2; }
mkdir -p "$GRADLE_HOME_CACHE" "$DEBUG_KEYSTORE_DIR"

SETUP='set -eu; G=/root/.gradle/dist/gradle-9.1.0; if [ ! -x $G/bin/gradle ]; then command -v unzip >/dev/null || (apt-get update -qq && apt-get install -y -qq unzip >/dev/null); mkdir -p /root/.gradle/dist; unzip -q /opt/gradle-dist.zip -d /root/.gradle/dist; fi; export PATH=$G/bin:$PATH'

echo "phase 1: fetch (network on)"
docker run --rm --platform linux/amd64 \
  -v "$HERE:/work" \
  -v "$LINUX_SDK:/opt/android-sdk" \
  -v "$GRADLE_HOME_CACHE:/root/.gradle" \
  -v "$DEBUG_KEYSTORE_DIR:/root/.android" \
  -v "$GRADLE_ZIP:/opt/gradle-dist.zip:ro" \
  -w /work \
  -e ANDROID_HOME=/opt/android-sdk \
  -e ANDROID_SDK_ROOT=/opt/android-sdk \
  "$IMAGE" sh -c "$SETUP; if [ ! -f gradlew ]; then gradle wrapper --gradle-version 9.1.0 --gradle-distribution-sha256-sum a17ddd85a26b6a7f5ddb71ff8b05fc5104c0202c6e64782429790c933686c806; fi; ./gradlew --version >/dev/null; ./gradlew --no-daemon :app:assembleDebug"

echo "phase 2: build (network none, offline)"
docker run --rm --platform linux/amd64 --network none \
  -v "$HERE:/work" \
  -v "$LINUX_SDK:/opt/android-sdk" \
  -v "$GRADLE_HOME_CACHE:/root/.gradle" \
  -v "$DEBUG_KEYSTORE_DIR:/root/.android" \
  -v "$GRADLE_ZIP:/opt/gradle-dist.zip:ro" \
  -w /work \
  -e ANDROID_HOME=/opt/android-sdk \
  -e ANDROID_SDK_ROOT=/opt/android-sdk \
  "$IMAGE" sh -c "$SETUP; ./gradlew --no-daemon --offline :app:assembleDebug"

APK="$HERE/app/build/outputs/apk/debug/app-debug.apk"
test -f "$APK" && echo "built: $APK" && shasum -a 256 "$APK"
