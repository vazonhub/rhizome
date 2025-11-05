# 1. Web (React)
```
wasm-pack build --target web --out-dir pkg/web
```

# 2. Flutter (Android + iOS)
```
cargo install cargo-ndk cargo-lipo
```

# Android
```
cargo ndk -t arm64-v8a -t armeabi-v7a build --release --features ffi
```

# iOS
```
cargo lipo --release --features ffi
```

# 3. Backend (Rust)
```
cargo add rhizome --path ./rhizome
```