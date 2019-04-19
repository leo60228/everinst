#!/bin/sh
docker run --rm -v "$(pwd):/tmp/everinst" --device /dev/fuse --cap-add SYS_ADMIN everinst-build:latest /tmp/everinst/build.sh
