rm -rf build
mkdir build
cargo build --all-targets --target-dir build --release
cd build/release
chmod +x ./sendai
./sendai --cfg prod
