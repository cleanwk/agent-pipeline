#!/usr/bin/env bash

set -euo pipefail

if test "$#" -ne 2; then
  echo "usage: $0 <app-bundle> <dmg>" >&2
  exit 2
fi

app_path="$1"
dmg_path="$2"

test -d "$app_path" || { echo "app bundle not found: $app_path" >&2; exit 1; }
test -f "$dmg_path" || { echo "DMG not found: $dmg_path" >&2; exit 1; }

signature_details="$(codesign -dv --verbose=4 "$app_path" 2>&1)"
if ! grep -Fq "Authority=Developer ID Application:" <<<"$signature_details"; then
  echo "app is not signed with a Developer ID Application certificate" >&2
  printf '%s\n' "$signature_details" >&2
  exit 1
fi
if grep -Fq "Signature=adhoc" <<<"$signature_details" || grep -Fq "TeamIdentifier=not set" <<<"$signature_details"; then
  echo "app has an ad-hoc or incomplete code signature" >&2
  exit 1
fi

codesign --verify --deep --strict --verbose=2 "$app_path"
spctl --assess --type execute --verbose=4 "$app_path"
xcrun stapler validate "$app_path"

icon_name="$(plutil -extract CFBundleIconFile raw "$app_path/Contents/Info.plist")"
test -n "$icon_name" || { echo "CFBundleIconFile is missing" >&2; exit 1; }
case "$icon_name" in
  *.icns) icon_path="$app_path/Contents/Resources/$icon_name" ;;
  *) icon_path="$app_path/Contents/Resources/$icon_name.icns" ;;
esac
test -f "$icon_path" || { echo "bundled icon not found: $icon_path" >&2; exit 1; }

hdiutil verify "$dmg_path"
