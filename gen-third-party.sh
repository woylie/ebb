#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2025 Mathias Polligkeit
#
# SPDX-License-Identifier: AGPL-3.0-or-later

set -euo pipefail

if ! cargo license --help &>/dev/null; then
  echo "Error: cargo license is not installed."
  echo "Install it with: cargo install --locked cargo-license"
  exit 1
fi

set -x

# Written to a temporary file first, so that a failed run leaves the previous
# contents in place rather than truncating them.
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

cargo license --color never --avoid-dev-deps > "$tmp"
mv "$tmp" THIRD_PARTY
