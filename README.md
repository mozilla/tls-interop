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





