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

# Build frame_decode
```
gcc -I/usr/include/ffmpeg -lavdevice -lavformat -lavfilter -lavcodec -lswresample -lswscale -lavutil frame_decode.c lib_frame_decode.c -g -o frame_decode
gcc -shared -fPIC   -I/usr/include/ffmpeg -lavdevice -lavformat -lavfilter -lavcodec -lswresample -lswscale -lavutil lib_frame_decode.c -o libframe_decode.so
```

# Build simple dll
```
gcc -shared -fPIC -o libsimpledll.so simple_dll.c
```

# TODO

* ~~support sqlite~~

* ~~sync mysql to sqlite~~

* ~~query date from sqlite~~

* ~~implement `init-video` interfacie~~

* ~~read param from environment~~

* ~~disable mysql if not needed~~

* ~~parse video size~~

* ~~insert to db~~

* ~~parse video meta info, insert to db~~

* ~~trans video steam~~

* ~~api version down to dir level~~

* ~~statistic interface~~

* copy cover file to main partition

* tab mark

* video snapshot interface
