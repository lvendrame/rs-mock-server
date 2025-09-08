#!/usr/bin/env bash
# scripts/gen-changelog.sh

 # 1) find all commits that modified Cargo.toml (in chronological order)
 bumps=$(git log --reverse --pretty=format:%H -- Cargo.toml)

# 2) if you never bumped yet, pretend the “initial” version is empty
prev=""

# 3) iterate over each bump
while IFS= read -r commit; do
  # extract the version value from Cargo.toml at that point
  version=$(git show "${commit}:Cargo.toml" \
    | sed -n 's/^version = "\([^"]*\)"/\1/p')
  # skip if version not found
  [[ -z "$version" ]] && continue

  echo "## $version ($(git show -s --format=%ci $commit | cut -d ' ' -f1))"
  echo

  if [[ -n "$prev" ]]; then
    git log --pretty="* %s (%an)" "$prev..$commit"
  else
    git log --pretty="* %s (%an)" "$commit"
  fi
  echo
  prev=$commit
done <<< "$bumps"

# 4) finally, list changes since last bump to HEAD
if [[ -n "$prev" ]]; then
  echo "## Unreleased"
  echo
  git log --pretty="* %s (%an)" "$prev..HEAD"
  echo
fi
