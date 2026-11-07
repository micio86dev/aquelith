# Reads Google's public Android package indexes and prints the archive size of each component we would
# need, for macOS hosts (or host-independent packages). Usage:
#   python3 -I measure-android-sizes.py repository2-3.xml sys-img2-3.xml
# Indexes: https://dl.google.com/android/repository/repository2-3.xml
#          https://dl.google.com/android/repository/sys-img/google_apis/sys-img2-3.xml
import sys, xml.etree.ElementTree as ET

WANT = {
    "cmdline-tools;latest", "platform-tools", "platforms;android-35",
    "build-tools;35.0.1", "emulator", "system-images;android-35;google_apis;arm64-v8a",
}

def archives(pkg):
    for a in pkg.iter():
        if not a.tag.endswith("archive"):
            continue
        host = [x.text for x in a if x.tag.endswith("host-os")]
        arch = [x.text for x in a if x.tag.endswith("host-arch")]
        size = [y.text for y in a.iter() if y.tag.endswith("size")]
        url = [y.text for y in a.iter() if y.tag.endswith("url")]
        yield (host[0] if host else None, arch[0] if arch else None,
               int(size[0]) if size else None, url[0] if url else None)

for index in sys.argv[1:]:
    root = ET.parse(index).getroot()
    for pkg in root.iter():
        if not pkg.tag.endswith("remotePackage"):
            continue
        path = pkg.get("path", "")
        if path not in WANT:
            continue
        for host, arch, size, url in archives(pkg):
            if host in ("macosx", None) and size:
                print(f"{path}\t{host}\t{arch}\t{size}\t{size/1048576:.1f} MiB\t{url}")
