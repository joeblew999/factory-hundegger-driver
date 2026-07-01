#!/usr/bin/env bash
# Fetch real-world .btlx files from public repos, for interop testing.
#
# We vendor only one small demo file (lignocam-hackathon.btlx) in git; the rest
# are other people's project data, so fetch them on demand rather than re-hosting.
# Requires the `gh` CLI (authenticated).
set -euo pipefail
cd "$(dirname "$0")"

get() { # owner/repo  path  outfile
  echo "-> $3"
  gh api "repos/$1/contents/$2" -H "Accept: application/vnd.github.raw" > "$3"
}

# Real Hundegger/Cambium-style machine exports (BTLx 2.0.0), child-element geometry:
get gramaziokohler/coding_architecture_fs26_focus_work fabrication/stencil_60x80.btlx eth-stencil_60x80.btlx
get MAS-dfab/easyhops                                   data/test.btlx                 easyhops-test.btlx

# NOTE: victorwhale/btlx-parser examples/sample.btlx is a hand-written toy that is
# NOT schema-conformant (it puts processing geometry in attributes; real files and
# the XSD use child elements). Kept out on purpose — don't model against it.

echo "done. (lignocam-hackathon.btlx is committed in-tree.)"
