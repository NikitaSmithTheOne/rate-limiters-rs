#!/bin/bash
echo "Pre-Release Hook"
git cliff -o CHANGELOG.md --tag "$1"
git add CHANGELOG.md