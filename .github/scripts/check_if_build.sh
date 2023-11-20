#!/bin/bash

# fetch main branch for comparison
git fetch origin main:main

cat <(git log -n 5)

base_commit=$(git rev-parse HEAD^)

echo "Current branch: $(git branch --show-current)"
echo "Current commit: $(git rev-parse HEAD)"
echo "Base commit: $base_commit"
echo "----------------------------------------"
# Compare the changes from the common ancestor to the current commit
changed_files=$(git diff --name-only $base_commit HEAD)
echo -e "Changed files: \n $changed_files"
echo "----------------------------------------"
check_files=("Cargo.toml" "Cargo.lock" "fly.toml" "Dockerfile")

for file in "${check_files[@]}"; do
    if [[ $changed_files == *"$file"* ]]; then
        echo "Set check_if_build=true >> \$GITHUB_OUTPUT"
        echo "check_if_build=true" >>$GITHUB_OUTPUT
        exit 0
    fi
done

if [[ $changed_files == *"src/"* ]]; then
    echo "Set check_if_build=true >> \$GITHUB_OUTPUT"
    echo "check_if_build=true" >>$GITHUB_OUTPUT
else
    echo "Set check_if_build=false >> \$GITHUB_OUTPUT"
    echo "check_if_build=false" >>$GITHUB_OUTPUT
fi
