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

# Install ffmpeg in fedora 41
```
sudo dnf install https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-41.noarch.rpm
sudo dnf install https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-41.noarch.rpm
sudo dnf group install multimedia
sudo dnf swap ffmpeg-free ffmpeg --allowerasing
sudo dnf install ffmpeg-devel
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
gcc -I/usr/include/ffmpeg frame_decode.c lib_frame_decode.c -g -o frame_decode -lavdevice -lavformat -lavfilter -lavcodec -lswresample -lswscale -lavutil
gcc -I/usr/include/ffmpeg -shared -fPIC lib_frame_decode.c -o libframe_decode.so -lavdevice -lavformat -lavfilter -lavcodec -lswresample -lswscale -lavutil
```

# Build simple dll
```
gcc -shared -fPIC -o libsimpledll.so simple_dll.c
```

# Install ffmpeg and codecs libs

## OpenSUSE

```shell
sudo zypper ar -cfp 90 'https://mirrors.aliyun.com/packman/suse/openSUSE_Tumbleweed' packman
sudo zypper refresh
sudo zypper dist-upgrade --from packman --allow-vendor-change
sudo zypper install --from packman ffmpeg gstreamer-plugins-{good,bad,ugly,libav} libavcodec-full vlc-codecs

sudo zypper install ffmpeg-7-libavcodec-devel ffmpeg-7-libavdevice-devel ffmpeg-7-libavfilter-devel ffmpeg-7-libavformat-devel ffmpeg-7-libavutil-devel ffmpeg-7-libpostproc-devel ffmpeg-7-libswresample-devel ffmpeg-7-libswscale-devel
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

* ~~copy cover file to main partition~~

* ~~image stream switch to main partition~~

* ~~tab mark~~

* video snapshot interface

* tab in dialog 

* ~~ui for search designation ~~