#!/bin/bash
# Trigger Windows build via GitHub Actions

echo "🚀 Triggering GitHub Actions Windows Build..."
echo ""

# Get current version from Cargo.toml
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
DATE=$(date +%Y%m%d)
TAG="v${VERSION}-windows-${DATE}"

echo "Creating tag: $TAG"
git tag "$TAG"

echo "Pushing to GitHub..."
git push origin "$TAG"

echo ""
echo "✅ GitHub Actions đang build..."
echo ""
echo "Xem tiến độ: https://github.com/QUOCVIETHIEU/QVCAD/actions"
echo ""
echo "Sau ~10-15 phút:"
echo "Tải file từ: https://github.com/QUOCVIETHIEU/QVCAD/releases"
