#!/bin/bash

set -e

if ! version=$(git describe --exact-match 2>/dev/null); then
    echo "Error: Not on a release tag. Please commit and tag a release first."
    echo "Current: $(git describe)"
    exit 1
fi

echo "Building release version: $version"

docker build . -t "agjini/crooner:${version}" -t "agjini/crooner:latest"
docker push "agjini/crooner:${version}"
docker push "agjini/crooner:latest"