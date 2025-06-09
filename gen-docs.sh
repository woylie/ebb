#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2025 Mathias Polligkeit
#
# SPDX-License-Identifier: AGPL-3.0-or-later

set -euo pipefail

if ! prettier --version &>/dev/null; then
  echo "Error: prettier is not installed."
  exit 1
fi

set -x

cargo run generate-docs > command_line_help.md
prettier ./command_line_help.md --write
