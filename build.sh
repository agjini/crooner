#!/bin/bash

version=$(git describe)

docker build . -t "agjini/crooner:${version}" -t "agjini/crooner:latest"
docker push "agjini/crooner:${version}"
docker push -t "agjini/crooner:latest"