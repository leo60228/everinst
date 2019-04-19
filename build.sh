#!/bin/bash -ex
cd /tmp/everinst
cargo deb
cd /tmp
wget https://raw.githubusercontent.com/AppImage/pkg2appimage/master/pkg2appimage
chmod +x pkg2appimage
bash -ex ./pkg2appimage /tmp/everinst/appimage.yml
mkdir -p /tmp/everinst/target/appimage/
cp -v ./out/*.AppImage /tmp/everinst/target/appimage/
