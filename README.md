[![Linux](https://github.com/dan4ik605743/hasher/actions/workflows/rust.yml/badge.svg)](https://github.com/dan4ik605743/hasher/actions/workflows/rust.yml)
# Hasher 

CLI utility for hashing the signature of a folder and checking for file changes in it.

## Compilation
```
git clone https://github.com/dan4ik605743/hasher
cd hasher
cargo build --release
```

## Usage (File hashing)
```
hasher -p /path to the folder for hashing files/ -b usize (Optional value for block size in hashing)
cat data.json
```

## Usage (Checking hashes)
```
hasher -p /path to the folder for hashing files -c data_check.json
```