#!/usr/bin/env bash
# Copy packaged workflow definitions into .github/workflows/ (required path for Actions).
# Needs a git push with the `workflow` OAuth scope the first time these files are added.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
mkdir -p "$ROOT/.github/workflows"
cp -v "$ROOT/ci/workflows/"*.yml "$ROOT/.github/workflows/"
echo "Copied. Commit and push (token needs 'workflow' scope):"
echo "  git add .github/workflows && git commit -m 'ci: enable GitHub Actions workflows' && git push"
