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

cargo license --color never --avoid-dev-deps > THIRD_PARTY
