# Install in wsl

```
sudo apt install clang
sudo apt install pkg-config
sudo apt install pkg-config-dev
sudo apt install libpkgconf-dev
sudo apt install libopenssl-dev
sudo apt install libssl-dev
sudo apt install libsqlite3-dev
```

# Build

```
cargo build --release
```

# TODO

* ~~support sqlite~~

* ~~sync mysql to sqlite~~

* ~~query date from sqlite~~

* ~~implement `init-video` interfacie~~

* read param from environment

* disable mysql if not needed
