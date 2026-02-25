#!/bin/bash
# Jacox Packager - Creates a portable deployment bundle

echo "🏗 Building Jacox release binary..."
cargo build --release

echo "📦 Packaging..."
mkdir -p dist
cp target/release/jacox dist/
cp config.yaml dist/
cp -r static dist/

echo "✅ Done! Your portable bundle is in the 'dist' folder."
echo "🚀 To deploy: zip 'dist' and move it to your server."
