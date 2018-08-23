#!/bin/bash

ROOT_DIR=$TRAVIS_BUILD_DIR/..

cd $ROOT_DIR
git clone https://chromium.googlesource.com/external/gyp
cd gyp
./setup.py build
sudo ./setup.py install
cd $ROOT_DIR
hg clone https://hg.mozilla.org/projects/nspr
hg clone https://hg.mozilla.org/projects/nss
cd nss
./build.sh
cd $ROOT_DIR
git clone https://github.com/google/boringssl.git
cd boringssl
git checkout -q 9af1edbe2201e6c6d58e5e484bf56281d8c751d9
mkdir build
cd build
cmake ..
make -j$(nproc)
cd $ROOT_DIR
git clone -q https://github.com/openssl/openssl.git
cd openssl
git checkout -q 9e6a32025e6e69949ad3e53a29a0b85f61f30b85
./config enable-external-tests
make -j$(nproc)
make install
cd $TRAVIS_BUILD_DIR

cargo test
