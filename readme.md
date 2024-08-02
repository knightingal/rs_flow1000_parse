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
# Config wsl
```
netsh interface portproxy add v4tov4 listenport=8082 listenaddress=0.0.0.0 connectport=8082 connectaddress=172.23.9.218
netsh interface portproxy add v4tov4 listenport=3002 listenaddress=0.0.0.0 connectport=3002 connectaddress=172.23.9.218
```

# TODO

* ~~support sqlite~~

* ~~sync mysql to sqlite~~

* ~~query date from sqlite~~

* ~~implement `init-video` interfacie~~

* ~~read param from environment~~

* ~~disable mysql if not needed~~
