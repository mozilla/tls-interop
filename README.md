
[![Build Status](https://travis-ci.org/jallmann/tls-interop.svg?branch=master)](https://travis-ci.org/jallmann/tls-interop)

Primitive TLS interop Harness
=============================

This program tests interop between two TLS stacks. In order to use it,
each stack needs to be wrapped in a BoringSSL runner-compatible
[shim](https://boringssl.googlesource.com/boringssl/+/master/ssl/test/PORTING.md).
The runner then runs the shims against each other in a variety (currently small)
of configurations).


Basic Execution Instructions
============================
The harness is run as:

```
tls_interop --client [shim-client] --server [shim-server] --rootdir=[path-to-key-files] --test-cases [test-case-descriptions]
```
For instance:

```
tls_interop --client ${NSS_ROOT}/dist/Darwin15.6.0_cc_64_DBG.OBJ/bin/nss_bogo_shim --server ${NSS_ROOT}/dist/Darwin15.6.0_cc_64_DBG.OBJ/bin/nss_bogo_shim --rootdir=${BORINGSSL_ROOT}/ssl/test/runner/ --test-cases cases.json
```

To swap client and server, you need to run it twice.


Cargo Test Instructions
============================
Some of the internal rust test cases run with "cargo test" assume readily built
versions of nss, boringssl and openssl being available in the parent diretory.
The NSS shim is expected to be found at "../dist/Debug/bin/nss_bogo_shim".  
The BoringSSL shim is expected to be found at "../boringssl/build/ssl/test/bssl_shim".  
The OpenSSL shim is expected to be found at "../openssl/tests/ossl_shim/ossl_shim".

NOTE: OpenSSL needs to be built with the "enable-external-tests" flag. Otherwise
the ossl_shim is not built.

All three default paths can be overwritten by setting the following environment variables:  
NSS_SHIM_PATH = ${NSS_ROOT}/bin/nss_bogo_shim  
BORING_ROOT_DIR = ${BORINGSSL_ROOT}  
OSSL_SHIM_PATH = ${OPENSSL_ROOT}/tests/ossl_shim/ossl_shim  

```
cargo test
```
Runs only a set of very basic connection tests between nss and the other two 
shims and additionally all test cases specified in the cases.json file, in each 
available combination of shims.
