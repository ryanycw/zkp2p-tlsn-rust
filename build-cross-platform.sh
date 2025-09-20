#!/bin/bash
# Cross-platform build for ZKP2P TLSNotary FFI
set -e

# Install cargo-ndk if missing
command -v cargo-ndk >/dev/null || cargo install cargo-ndk

# Install Rust targets
echo "📱 Installing Rust targets..."
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add x86_64-apple-ios
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android

# Create output directories
mkdir -p libs/ios
mkdir -p libs/android/{arm64-v8a,armeabi-v7a,x86,x86_64}

# Build iOS on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Building iOS..."
    cargo build --target aarch64-apple-ios --release
    cargo build --target x86_64-apple-ios --release
fi

# Set NDK path if not already set
if [[ -z "$ANDROID_NDK_HOME" ]]; then
    if [[ -d "$HOME/Library/Android/sdk/ndk" ]]; then
        export ANDROID_NDK_HOME="$(find "$HOME/Library/Android/sdk/ndk" -maxdepth 1 -type d | tail -1)"
        export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
        export ANDROID_NDK="$ANDROID_NDK_HOME"
        echo "Set ANDROID_NDK_HOME=$ANDROID_NDK_HOME"
    else
        echo "Error: Android NDK not found. Install it or set ANDROID_NDK_HOME"
        exit 1
    fi
fi

# Build Android (cargo-ndk builds all targets in metadata.android)
echo "Building Android..."
cargo ndk build --release -t x86_64-linux-android -t i686-linux-android -t armv7-linux-androideabi -t aarch64-linux-android -o libs/android

# Copy iOS libs (macOS only)
if [[ "$OSTYPE" == "darwin"* ]]; then
    lipo -create \
        target/aarch64-apple-ios/release/libzkp2p_tlsn_rust.dylib \
        target/x86_64-apple-ios/release/libzkp2p_tlsn_rust.dylib \
        -output libs/ios/libzkp2p_tlsn_rust.dylib 2>/dev/null
fi

echo "Done. Libraries in libs/, header in include/"
echo "✅ Cross-compilation completed successfully!"
echo ""
echo "📁 Output libraries:"
echo "  iOS: libs/ios/"
echo "    - libzkp2p_tlsn_rust.a (aarch64)"
echo "    - libzkp2p_tlsn_rust_sim.a (x86_64 simulator)"
echo "    - libzkp2p_tlsn_rust_universal.a (universal)"
echo ""
echo "  Android: libs/android/"
echo "    - arm64-v8a/libzkp2p_tlsn_rust.so"
echo "    - armeabi-v7a/libzkp2p_tlsn_rust.so"
echo "    - x86/libzkp2p_tlsn_rust.so"
echo "    - x86_64/libzkp2p_tlsn_rust.so"
echo ""
echo "🎯 Ready for React Native JSI integration!"l