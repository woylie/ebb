#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2025 Mathias Polligkeit
#
# SPDX-License-Identifier: AGPL-3.0-or-later

# Bumps the version in Cargo.toml, Cargo.lock and the changelog, and for a
# release also in the install commands of the README and the book.
#
# Usage: ./bump.sh major|minor|patch|pre|<version>

set -euo pipefail
export LC_ALL=C

cd "$(dirname "$0")"

# semantic versions without build metadata
number='(0|[1-9][0-9]*)'
identifier='(0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)'
version_pattern="^${number}\.${number}\.${number}(-${identifier}(\.${identifier})*)?$"

docs=(README.md book/src/installation.md)
changelog=CHANGELOG.md
files=(Cargo.toml Cargo.lock "$changelog" "${docs[@]}")

tmpfile=$(mktemp)
backups=$(mktemp -d)
state=checking

# Puts every file back as it was when a run stops after the first write.
cleanup() {
  local file
  if [[ $state == writing ]]; then
    for file in "${files[@]}"; do
      cat "$backups/$file" >"$file"
    done
    echo "The bump did not finish. All files are as they were before." >&2
  fi
  rm -rf "$tmpfile" "$backups"
}
trap cleanup EXIT

usage() {
  echo "Usage: ./bump.sh major|minor|patch|pre|<version>" >&2
}

# Runs a command with file $1 as its last argument and writes the output back
# into $1. Writing rather than moving the temporary file keeps $1's permissions.
rewrite() {
  local file=$1
  shift
  "$@" "$file" >"$tmpfile" && cat "$tmpfile" >"$file"
}

# Prints 1, 0 or -1 as the identifier $1 is greater than, equal to or less than
# $2.
compare_identifiers() {
  local a=$1 b=$2
  if [[ $a =~ ^[0-9]+$ && $b =~ ^[0-9]+$ ]]; then
    if ((10#$a > 10#$b)); then echo 1; elif ((10#$a < 10#$b)); then echo -1; else echo 0; fi
  elif [[ $a =~ ^[0-9]+$ ]]; then
    echo -1
  elif [[ $b =~ ^[0-9]+$ ]]; then
    echo 1
  elif [[ $a > $b ]]; then
    echo 1
  elif [[ $a < $b ]]; then
    echo -1
  else
    echo 0
  fi
}

# Prints 1, 0 or -1 as the dot-separated identifiers $1 are greater than, equal
# to or less than $2.
compare_lists() {
  local -a a b
  local i result
  IFS=. read -r -a a <<<"$1"
  IFS=. read -r -a b <<<"$2"

  for ((i = 0; i < ${#a[@]} && i < ${#b[@]}; i++)); do
    result=$(compare_identifiers "${a[i]}" "${b[i]}")
    if [[ $result != 0 ]]; then
      echo "$result"
      return
    fi
  done

  if ((${#a[@]} > ${#b[@]})); then echo 1; elif ((${#a[@]} < ${#b[@]})); then echo -1; else echo 0; fi
}

# Succeeds if version $1 is greater than version $2 by the precedence rules of
# semantic versioning.
greater() {
  local a_pre='' b_pre='' core
  [[ $1 == *-* ]] && a_pre=${1#*-}
  [[ $2 == *-* ]] && b_pre=${2#*-}

  core=$(compare_lists "${1%%-*}" "${2%%-*}")
  if [[ $core != 0 ]]; then
    [[ $core == 1 ]]
    return
  fi

  # a release is greater than its pre-releases
  if [[ -z $a_pre || -z $b_pre ]]; then
    [[ -z $a_pre && -n $b_pre ]]
    return
  fi

  [[ $(compare_lists "$a_pre" "$b_pre") == 1 ]]
}

next_version() {
  local major minor patch
  IFS=. read -r major minor patch <<<"$1"

  case $2 in
  major) echo "$((10#$major + 1)).0.0" ;;
  minor) echo "$major.$((10#$minor + 1)).0" ;;
  patch) echo "$major.$minor.$((10#$patch + 1))" ;;
  esac
}

next_pre_version() {
  if [[ $1 =~ ^([0-9]+\.[0-9]+\.[0-9]+-(.*[^0-9])?)([0-9]+)$ ]]; then
    echo "${BASH_REMATCH[1]}$((10#${BASH_REMATCH[3]} + 1))"
  fi
}

bump_cargo() {
  local new_version=$1
  # shellcheck disable=SC2016 # awk expands $0 in its own program
  if ! rewrite Cargo.toml awk -v version="$new_version" \
    '/^\[/ { in_package = ($0 == "[package]") }
     in_package && !done && /^version = "/ { print "version = \"" version "\""; done = 1; next }
     { print }
     END { exit !done }'; then
    echo "Cargo.toml has no line of the form version = \"…\" in its [package] table. Write the version that way, then run the script again." >&2
    exit 1
  fi

  if ! cargo update --workspace --offline --quiet; then
    echo "cargo could not update Cargo.lock." >&2
    exit 1
  fi
}

bump_changelog() {
  local new_version=$1 today link_line unreleased_link='' release_link=''
  today=$(date -u +'%Y-%m-%d')
  link_line=$(grep -E '^\[[^]]+\]: .+/compare/.+\.\.\.HEAD$' "$changelog" | head -n 1 || true)

  if [[ -n $link_line && $link_line =~ ^(\[[^]]+\]):\ (.+/compare/)(.+)\.\.\.HEAD$ ]]; then
    local label=${BASH_REMATCH[1]} base=${BASH_REMATCH[2]} old_tag=${BASH_REMATCH[3]}
    local new_tag=${old_tag%%[0-9]*}$new_version
    unreleased_link="$label: $base$new_tag...HEAD"
    release_link="[$new_version]: $base$old_tag...$new_tag"
  else
    echo "No unreleased link definition in $changelog. Skipping links." >&2
  fi

  # shellcheck disable=SC2016 # awk expands $0 in its own program
  rewrite "$changelog" awk -v heading="## [$new_version] - $today" \
    -v old_link="$link_line" \
    -v new_unreleased="$unreleased_link" \
    -v new_release="$release_link" \
    '/^## \[?Unreleased\]?$/ && !inserted { print; print ""; print heading; inserted = 1; next }
     old_link != "" && $0 == old_link { print new_unreleased; print new_release; next }
     { print }'
}

bump_docs() {
  local new_version=$1 file
  for file in "${docs[@]}"; do
    rewrite "$file" sed -E \
      "s#(github:woylie/ebb/|--branch |version=)v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?#\\1v${new_version}#g"
  done
}

if [[ $# -ne 1 ]]; then
  usage
  exit 1
fi

if [[ $1 == -h || $1 == --help ]]; then
  usage
  exit 0
fi

if ! command -v cargo >/dev/null; then
  echo "cargo is not installed. Run the script in the dev shell (nix develop)." >&2
  exit 1
fi

package_id=$(cargo pkgid --offline)
current=${package_id##*[#@]}

if [[ ! $current =~ $version_pattern ]]; then
  echo "The version in Cargo.toml is not a semantic version without build metadata. Fix it by hand, then run the script again. Found: \"$current\"" >&2
  exit 1
fi

case $1 in
major | minor | patch)
  if [[ $current == *-* ]]; then
    echo "Cannot bump the $1 part of the pre-release version $current. Use \"./bump.sh pre\" for the next pre-release, or pass the version, e.g. ./bump.sh ${current%%-*}." >&2
    exit 1
  fi
  new=$(next_version "$current" "$1")
  ;;
pre)
  if [[ $current != *-* ]]; then
    echo "$current is not a pre-release version. Pass the version instead, e.g. ./bump.sh $(next_version "$current" minor)-rc.1." >&2
    exit 1
  fi
  new=$(next_pre_version "$current")
  if [[ -z $new ]]; then
    echo "Cannot derive the next pre-release version from $current. Pass the version instead." >&2
    exit 1
  fi
  ;;
*)
  if [[ ! $1 =~ $version_pattern ]]; then
    echo "'$1' is not a semantic version without build metadata." >&2
    usage
    exit 1
  fi
  new=$1
  ;;
esac

if ! greater "$new" "$current"; then
  echo "$new is not greater than the current version $current. Pass a greater version." >&2
  exit 1
fi

if ! grep -qE '^## \[?Unreleased\]?$' "$changelog"; then
  echo "$changelog has no \"## [Unreleased]\" heading. Add one above the latest release, then run the script again." >&2
  exit 1
fi

for file in "${files[@]}"; do
  if [[ ! -f $file ]]; then
    echo "$file does not exist." >&2
    exit 1
  fi
done

echo "Current version: $current"
echo "New version: $new"

for file in "${files[@]}"; do
  mkdir -p "$backups/$(dirname "$file")"
  cp "$file" "$backups/$file"
done
state=writing

bump_cargo "$new"
bump_changelog "$new"

if [[ $new == *-* ]]; then
  echo "Pre-release: the install commands keep pointing at the last release."
else
  bump_docs "$new"
fi

state=finished
