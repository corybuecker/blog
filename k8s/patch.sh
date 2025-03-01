#!/bin/bash

set -e
set -o pipefail

IFS=$'\n'

if command -v gsed &> /dev/null; then
  gsed -i "s/version: \"[0-9]\{10\}\"/version: \"$(date +%s)\"/" k8s/deployment.yaml
else
  sed -i "s/version: \"[0-9]\{10\}\"/version: \"$(date +%s)\"/" k8s/deployment.yaml
fi
