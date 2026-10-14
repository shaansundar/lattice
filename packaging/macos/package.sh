#!/usr/bin/env bash
# Build, sign and (optionally) notarize the macOS release artifacts:
#
#   $DIST/lattice-<version>-macos-<arch>.dmg          Lattice.app on a drag-to-Applications DMG
#   $DIST/lattice-cli-<version>-macos-<arch>.zip      the headless CLI
#
# Usage: packaging/macos/package.sh [--arch universal|aarch64|x86_64] [--skip-build]
#
# Signing (env):
#   MACOS_SIGN_IDENTITY   codesign identity (name or SHA-1). Default "-" = ad-hoc (local testing;
#                         Gatekeeper will reject the result on other Macs). In CI, import-cert.sh sets it.
#   MACOS_KEYCHAIN        keychain holding the identity (optional)
# Notarization (env; all three needed, and a real identity):
#   APPLE_ID, APPLE_PASSWORD (app-specific password), APPLE_TEAM_ID
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/macos"

ARCH=universal
SKIP_BUILD=0
while [ $# -gt 0 ]; do
  case "$1" in
    --arch) ARCH="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    -h | --help) sed -n '2,16p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
case "$ARCH" in
  universal) TARGETS=(aarch64-apple-darwin x86_64-apple-darwin) ;;
  aarch64) TARGETS=(aarch64-apple-darwin) ;;
  x86_64) TARGETS=(x86_64-apple-darwin) ;;
  *) echo "unknown --arch $ARCH" >&2; exit 2 ;;
esac

# Keep in sync with LSMinimumSystemVersion in Info.plist.in.
export MACOSX_DEPLOYMENT_TARGET=11.0
IDENTITY="${MACOS_SIGN_IDENTITY:--}"
SHORT_VERSION="${VERSION%%-*}"
WORK="$CARGO_TARGET_DIR/macos-package"
APP="$WORK/Lattice.app"
DMG="$DIST/lattice-$VERSION-macos-$ARCH.dmg"
CLI_ZIP="$DIST/lattice-cli-$VERSION-macos-$ARCH.zip"

NOTARIZE=0
if [ "$IDENTITY" = "-" ]; then
  warn "macOS: ad-hoc signing (no MACOS_SIGN_IDENTITY); artifacts are not notarized"
elif [ -n "${APPLE_ID:-}" ] && [ -n "${APPLE_PASSWORD:-}" ] && [ -n "${APPLE_TEAM_ID:-}" ]; then
  NOTARIZE=1
else
  warn "macOS: APPLE_ID / APPLE_PASSWORD / APPLE_TEAM_ID incomplete; signed but not notarized"
fi

echo "==> Lattice $VERSION for macOS ($ARCH), identity: $IDENTITY, notarize: $NOTARIZE"

# ---- build -------------------------------------------------------------------------------------
if [ "$SKIP_BUILD" = 0 ]; then
  args=()
  for t in "${TARGETS[@]}"; do args+=(--target "$t"); done
  (cd "$ROOT" && cargo build --release --locked -p lattice -p lattice-cli "${args[@]}")
fi

rm -rf "$WORK"
mkdir -p "$WORK/bin"
for bin in lattice lattice-cli; do
  inputs=()
  for t in "${TARGETS[@]}"; do inputs+=("$CARGO_TARGET_DIR/$t/release/$bin"); done
  lipo -create -output "$WORK/bin/$bin" "${inputs[@]}"
  lipo -info "$WORK/bin/$bin"
done

# ---- signing helpers ---------------------------------------------------------------------------
sign() {
  # Hardened runtime + secure timestamp for a real identity; ad-hoc can't be timestamped.
  local ts=(--timestamp)
  [ "$IDENTITY" = "-" ] && ts=(--timestamp=none)
  local kc=()
  [ -n "${MACOS_KEYCHAIN:-}" ] && kc=(--keychain "$MACOS_KEYCHAIN")
  codesign --force --sign "$IDENTITY" ${kc[@]+"${kc[@]}"} "${ts[@]}" "$@"
}

notarize() {
  local file="$1" out id status
  echo "==> notarizing $(basename "$file") (this can take a few minutes)"
  out="$(xcrun notarytool submit "$file" --apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" --wait --timeout 1h --output-format json)" || true
  echo "$out"
  id="$(printf '%s' "$out" | plutil -extract id raw -o - - 2>/dev/null || true)"
  status="$(printf '%s' "$out" | plutil -extract status raw -o - - 2>/dev/null || true)"
  if [ "$status" != "Accepted" ]; then
    if [ -n "$id" ]; then
      xcrun notarytool log "$id" --apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" --team-id "$APPLE_TEAM_ID" || true
    fi
    echo "error: notarization of $(basename "$file") failed (status: ${status:-unknown})" >&2
    exit 1
  fi
}

# ---- Lattice.app ----------------------------------------------------------------------------
echo "==> assembling $APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
# Executable and icon carry the display name (CFBundleExecutable / CFBundleIconFile).
cp "$WORK/bin/lattice" "$APP/Contents/MacOS/Lattice"
cp "$ROOT/assets/app-icon/lattice.icns" "$APP/Contents/Resources/Lattice.icns"
# Licences of the embedded craft-fonts fonts (only when built with CRAFT_FONTS_DIR).
copy_font_licences "$APP/Contents/Resources"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/$SHORT_VERSION/g" \
  -e "s/@BUILD_SHA@/${LATTICE_BUILD_SHA:-unknown}/g" \
  "$HERE/Info.plist.in" >"$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"
printf 'APPL????' >"$APP/Contents/PkgInfo"

# ---- assistant runtime (llama.cpp's llama-server) ---------------------------------------------
# LATTICE_LLAMA_DIR_AARCH64 / LATTICE_LLAMA_DIR_X86_64: unpacked llama.cpp macOS release folders (llama-server,
# lib*.dylib, LICENSE), one per target arch; LATTICE_LLAMA_DIR stands in for a single-arch build. They go to
# Contents/Helpers/assistant (crates/assistant/src/install.rs). Without them the app ships without the runtime and
# the Assistant pane says so. Models are never bundled: the app downloads them (docs/assistant.md).
bundle_assistant() {
  local dest="$APP/Contents/Helpers/assistant" dirs=() t var d f name
  for t in "${TARGETS[@]}"; do
    case "$t" in aarch64-*) var=LATTICE_LLAMA_DIR_AARCH64 ;; *) var=LATTICE_LLAMA_DIR_X86_64 ;; esac
    d="${!var:-}"
    if [ -z "$d" ] && [ ${#TARGETS[@]} = 1 ]; then d="${LATTICE_LLAMA_DIR:-}"; fi
    if [ -z "$d" ]; then
      warn "macOS: no llama.cpp runtime for $t (set $var); the assistant won't be available in this build"
      return 0
    fi
    [ -x "$d/llama-server" ] || { echo "error: $d/llama-server not found" >&2; exit 1; }
    dirs+=("$d")
  done
  mkdir -p "$dest"
  # The server and the libraries it loads (@rpath closure from otool, version symlinks kept), from the first arch.
  # Only code goes in Helpers (codesign treats everything there as code); the licence goes to Resources.
  local src="${dirs[0]}" todo="llama-server" done_list=" " n dep
  while [ -n "$todo" ]; do
    n="${todo%% *}"
    if [ "$n" = "$todo" ]; then todo=""; else todo="${todo#* }"; fi
    case "$done_list" in *" $n "*) continue ;; esac
    done_list="$done_list$n "
    [ -e "$src/$n" ] || { echo "error: $src/$n (needed by llama-server) not found" >&2; exit 1; }
    cp -P "$src/$n" "$dest/"
    # Follow version symlinks (libggml.0.dylib -> libggml.0.26.0.dylib) to the real file.
    while [ -L "$src/$n" ]; do
      n="$(readlink "$src/$n")"
      cp -P "$src/$n" "$dest/"
    done
    for dep in $(otool -L "$src/$n" | awk '/@rpath\//{sub("@rpath/", "", $1); print $1}'); do
      todo="$todo${todo:+ }$dep"
    done
  done
  cp "$src/LICENSE" "$APP/Contents/Resources/LICENSE-llama.cpp.txt"
  # ...merged with the other arches file by file for a universal build.
  if [ ${#dirs[@]} -gt 1 ]; then
    for f in "$dest/llama-server" $(find "$dest" -name '*.dylib' -type f); do
      name="$(basename "$f")"
      local inputs=()
      for d in "${dirs[@]}"; do inputs+=("$d/$name"); done
      lipo -create -output "$f" "${inputs[@]}"
    done
  fi
  # Ad-hoc signatures carry no Team ID, so the hardened runtime's library validation would stop llama-server
  # loading its own libraries ("different Team IDs"): only a real identity (which notarization requires anyway)
  # gets --options runtime. All files then share one Team ID.
  local opts=()
  [ "$IDENTITY" != "-" ] && opts=(--options runtime)
  for f in $(find "$dest" -name '*.dylib' -type f) "$dest/llama-server"; do
    sign ${opts[@]+"${opts[@]}"} "$f"
  done
  "$dest/llama-server" --version >/dev/null 2>&1 || warn "macOS: the bundled llama-server didn't run on this Mac (wrong arch?)"
}
bundle_assistant

# Sign inside-out: nested code first, then the bundle itself (no --deep on the final signature).
# Nested code: the assistant runtime (signed above) and the main executable.
sign --options runtime --entitlements "$HERE/entitlements.plist" "$APP/Contents/MacOS/Lattice"
sign --options runtime --entitlements "$HERE/entitlements.plist" "$APP"
codesign --verify --strict --deep --verbose=2 "$APP"

if [ "$NOTARIZE" = 1 ]; then
  ditto -c -k --keepParent "$APP" "$WORK/Lattice-notarize.zip"
  notarize "$WORK/Lattice-notarize.zip"
  xcrun stapler staple "$APP"
  xcrun stapler validate "$APP"
  spctl --assess --type execute -vvv "$APP"
fi

# ---- DMG ---------------------------------------------------------------------------------------
echo "==> building $DMG"
STAGE="$WORK/dmg"
mkdir -p "$STAGE"
ditto "$APP" "$STAGE/Lattice.app"
ln -s /Applications "$STAGE/Applications"
# Finder window layout: background, icon size and positions (packaging/macos/dmg/README.md).
mkdir -p "$STAGE/.background"
cp "$HERE/dmg/background.tiff" "$STAGE/.background/background.tiff"
cp "$HERE/dmg/dmg-layout.DS_Store" "$STAGE/.DS_Store"
rm -f "$DMG" "$WORK/raw.dmg"
# makehybrid + convert builds the image without attaching a device, unlike `create -srcfolder`,
# which is flaky on CI runners ("Resource busy") and hangs in sandboxed sessions.
# The volume name has no version: .DS_Store finds the background through an alias that includes it.
hdiutil makehybrid -hfs -hfs-volume-name "Lattice" -hfs-openfolder "$STAGE" -o "$WORK/raw.dmg" "$STAGE"
hdiutil convert "$WORK/raw.dmg" -format UDZO -imagekey zlib-level=9 -o "$DMG"
rm -f "$WORK/raw.dmg"
sign "$DMG"
codesign --verify --strict --verbose=2 "$DMG"
if [ "$NOTARIZE" = 1 ]; then
  notarize "$DMG"
  xcrun stapler staple "$DMG"
  xcrun stapler validate "$DMG"
  spctl --assess --type open --context context:primary-signature -vvv "$DMG"
fi

# ---- CLI ---------------------------------------------------------------------------------------
echo "==> building $CLI_ZIP"
CLI_DIR="$WORK/lattice-cli-$VERSION-macos-$ARCH"
mkdir -p "$CLI_DIR"
cp "$WORK/bin/lattice-cli" "$CLI_DIR/"
copy_docs "$CLI_DIR"
sign --options runtime "$CLI_DIR/lattice-cli"
codesign --verify --strict --verbose=2 "$CLI_DIR/lattice-cli"
rm -f "$CLI_ZIP"
ditto -c -k --keepParent "$CLI_DIR" "$CLI_ZIP"
# A bare Mach-O can't carry a stapled ticket; Gatekeeper looks the notarization up online.
if [ "$NOTARIZE" = 1 ]; then notarize "$CLI_ZIP"; fi

"$WORK/bin/lattice-cli" --version
echo "==> done"
ls -lh "$DMG" "$CLI_ZIP"
