#!/bin/bash
set -e

echo "Building OpenHousePopulator..."
cargo build --release

echo "Creating bin/ directory..."
mkdir -p bin

echo "Copying binary..."
cp target/release/openhousepopulator bin/OpenHousePopulator

echo "Copying configuration and resources..."
cp settings.toml bin/settings.toml
cp -r res bin/res

echo ""
echo "Setup complete. Run the tool with:"
echo "  cd bin"
echo "  ./OpenHousePopulator --help"
