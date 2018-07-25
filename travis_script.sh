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
mkdir build
cd build
cmake ..
make
cd $ROOT_DIR
git clone -q https://github.com/openssl/openssl.git
cd openssl
./config enable-external-tests
make
make install
cd $TRAVIS_BUILD_DIR

cargo test
