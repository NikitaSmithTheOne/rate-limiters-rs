#!/bin/bash
echo "Pre-Release Hook"
git cliff -o CHANGELOG.md
git add CHANGELOG.md