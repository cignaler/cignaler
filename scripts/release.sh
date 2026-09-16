#!/usr/bin/env bash
#
# Cuts a release: bumps the version everywhere it is written down, proves the
# tree still builds and passes, tags, pushes, and waits for the release
# workflow.
#
# The version lives in three files that must agree -- package.json,
# src-tauri/Cargo.toml and src-tauri/tauri.conf.json -- plus Cargo.lock, which
# is derived. Bumping them by hand is how you end up with a DMG whose
# Info.plist disagrees with its filename.
#
# Publishing stays manual by default. release.yml sets releaseDraft: true so
# the artifacts can be looked at before anyone can download them; pass
# --publish to have this script flip the draft once the run is green.
#
# Usage:
#   scripts/release.sh 0.0.4              # tag, push, wait, leave a draft
#   scripts/release.sh 0.0.4 --publish    # ... and publish it when green
#   scripts/release.sh 0.0.4 --dry-run    # bump and test, change nothing else

set -euo pipefail

cd "$(dirname "$0")/.."

VERSION="${1:-}"
PUBLISH=false
DRY_RUN=false

for arg in "${@:2}"; do
  case "$arg" in
    --publish) PUBLISH=true ;;
    --dry-run) DRY_RUN=true ;;
    *) echo "unknown option: $arg" >&2; exit 1 ;;
  esac
done

die() { echo "error: $*" >&2; exit 1; }
step() { echo; echo "--- $*"; }

[ -n "$VERSION" ] || die "usage: $0 <version> [--publish] [--dry-run]"

# Reject a leading v now rather than producing the tag vv0.0.4 later.
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] ||
  die "version must be bare semver like 0.0.4 (no leading v), got: $VERSION"

TAG="v$VERSION"

step "1/7  preflight"

command -v gh >/dev/null || die "gh is not installed"
gh auth status >/dev/null 2>&1 || die "gh is not authenticated; run: gh auth login"

BRANCH=$(git rev-parse --abbrev-ref HEAD)
[ "$BRANCH" = "main" ] || die "releases are cut from main, currently on $BRANCH"

# Untracked files are fine -- scratch notes should not block a release -- but
# modified tracked files would silently ride along in the release commit.
git diff --quiet || die "you have unstaged changes; commit or stash them"
git diff --cached --quiet || die "you have staged changes; commit or stash them"

git fetch --quiet origin main
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] ||
  die "main is not in sync with origin/main; pull or push first"

git rev-parse --verify --quiet "refs/tags/$TAG" >/dev/null &&
  die "tag $TAG already exists locally"
git ls-remote --exit-code --tags origin "$TAG" >/dev/null 2>&1 &&
  die "tag $TAG already exists on origin"

CURRENT=$(node -p "require('./package.json').version")
echo "    $CURRENT -> $VERSION"

# The seven Apple secrets gate the macOS leg. release.yml checks them too, but
# it does so eleven minutes and one pushed tag too late to be useful.
# Fetched once, into a variable. Piping it into `grep -q` per secret looks
# tidier and is quietly broken: grep exits at the first match, gh takes SIGPIPE
# on the closed pipe, and under `set -o pipefail` that failure becomes the
# pipeline's status -- so a secret that is present gets reported missing,
# depending on how the race lands. A herestring has no pipe to break.
SECRETS=$(gh secret list --json name -q '.[].name')

MISSING=()
for secret in APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY \
              KEYCHAIN_PASSWORD APPLE_API_KEY APPLE_API_ISSUER APPLE_API_KEY_P8_BASE64; do
  grep -qx "$secret" <<<"$SECRETS" || MISSING+=("$secret")
done
[ ${#MISSING[@]} -eq 0 ] || die "signing secrets not set: ${MISSING[*]} (see docs/RELEASE.md)"
echo "    signing secrets present"

# Not fatal: the cask job failing does not spoil the DMG, and the tap is a
# separate piece of setup. Say so once here instead of leaving a red X to
# rediscover later.
if ! grep -qx HOMEBREW_TAP_TOKEN <<<"$SECRETS"; then
  echo "    note: HOMEBREW_TAP_TOKEN unset, so the Homebrew cask job will fail."
  echo "          The DMG is unaffected. See docs/RELEASE.md section 3."
fi

step "2/7  bump version in package.json, Cargo.toml, tauri.conf.json"

# JSON through node, so formatting and escaping stay the tool's problem.
node -e '
  const fs = require("fs");
  const version = process.argv[1];
  for (const file of ["package.json", "src-tauri/tauri.conf.json"]) {
    const json = JSON.parse(fs.readFileSync(file, "utf8"));
    json.version = version;
    fs.writeFileSync(file, JSON.stringify(json, null, 2) + "\n");
    console.log(`    ${file}`);
  }
' "$VERSION"

# Only the [package] version, which is the first `version =` in the file --
# the dependency versions further down must not move.
perl -i -pe 'if (!$done && s/^version = ".*"$/version = "'"$VERSION"'"/) { $done = 1 }' \
  src-tauri/Cargo.toml
echo "    src-tauri/Cargo.toml"

grep -q "^version = \"$VERSION\"$" src-tauri/Cargo.toml ||
  die "Cargo.toml version did not take; check it by hand"

step "3/7  build the sidecar and refresh Cargo.lock"

# tauri_build::build() asserts the externalBin sidecar exists, so it has to be
# staged before anything else compiles. This also rewrites Cargo.lock with the
# new package version.
node scripts/stage-native-host.mjs debug

step "4/7  test"

pnpm install --frozen-lockfile
pnpm run test:run
pnpm run check
cargo test --manifest-path src-tauri/Cargo.toml

if [ "$DRY_RUN" = true ]; then
  step "dry run: stopping here"
  echo "Version files and Cargo.lock are modified but nothing was committed."
  echo "Undo with: git checkout -- package.json src-tauri/"
  exit 0
fi

step "5/7  commit and tag $TAG"

git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -q -m "release: $TAG"
git tag -a "$TAG" -m "Cignaler $TAG"
git log --oneline -1

step "6/7  push, and wait for the release workflow"

git push --quiet origin main
git push --quiet origin "$TAG"
echo "    pushed main and $TAG"

# The run does not exist the instant the tag lands; give it a moment to appear.
RUN_ID=""
for _ in $(seq 1 30); do
  RUN_ID=$(gh run list --workflow=release.yml --branch "$TAG" \
             --limit 1 --json databaseId -q '.[0].databaseId' 2>/dev/null || true)
  [ -n "$RUN_ID" ] && break
  sleep 5
done
[ -n "$RUN_ID" ] || die "release workflow never started for $TAG; check the Actions tab"

echo "    watching run $RUN_ID"
if ! gh run watch "$RUN_ID" --exit-status; then
  die "release run failed; the draft (if any) is left alone: $(gh run view "$RUN_ID" --json url -q .url)"
fi

step "7/7  release"

if [ "$PUBLISH" = true ]; then
  gh release edit "$TAG" --draft=false >/dev/null
  echo "    published: $(gh release view "$TAG" --json url -q .url)"
  echo "    this fires homebrew.yml, which updates the cask"
else
  gh release view "$TAG" || true
  echo
  echo "Draft only -- it is not visible on the releases page yet."
  echo "Check the artifacts, then publish with:"
  echo
  echo "    gh release edit $TAG --draft=false"
fi
