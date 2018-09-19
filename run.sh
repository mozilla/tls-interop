#!/usr/bin/env bash
BASE_DIR=$(cd $(dirname $0); pwd -P)
CASE_FILE=$2
#export RUST_LOG=debug

run_boring_server() {
  cargo run -- \
  --client $BASE_DIR/../dist/Debug/bin/nss_bogo_shim \
  --server $BASE_DIR/../boringssl/build/ssl/test/bssl_shim \
  --rootdir $BASE_DIR/../boringssl/ssl/test/runner/ \
  --test-cases $BASE_DIR/$CASE_FILE \
  --client-writes-first
}

run_boring_client() {
  cargo run -- \
  --client $BASE_DIR/../boringssl/build/ssl/test/bssl_shim \
  --server $BASE_DIR/../dist/Debug/bin/nss_bogo_shim \
  --rootdir $BASE_DIR/../boringssl/ssl/test/runner/ \
  --test-cases $BASE_DIR/$CASE_FILE
}

run_ossl_server() {
  cargo run -- \
  --client $BASE_DIR/../dist/Debug/bin/nss_bogo_shim \
  --server $BASE_DIR/../openssl/test/ossl_shim/ossl_shim \
  --rootdir $BASE_DIR/../boringssl/ssl/test/runner/ \
  --test-cases $BASE_DIR/$CASE_FILE \
  --client-writes-first \
  --force-IPv4
}

run_ossl_client() {
  cargo run -- \
  --client $BASE_DIR/../openssl/test/ossl_shim/ossl_shim \
  --server $BASE_DIR/../dist/Debug/bin/nss_bogo_shim \
  --rootdir $BASE_DIR/../boringssl/ssl/test/runner/ \
  --test-cases $BASE_DIR/$CASE_FILE \
  --force-IPv4
}

run_loopback() {
  cargo run -- \
  --client $BASE_DIR/../dist/Debug/bin/nss_bogo_shim \
  --server $BASE_DIR/../dist/Debug/bin/nss_bogo_shim \
  --rootdir $BASE_DIR/../boringssl/ssl/test/runner/ \
  --test-cases $BASE_DIR/$CASE_FILE
}

case "$1" in
  "boring_server")
      run_boring_server
      ;;
  "boring_client")
      run_boring_client
      ;;
  "ossl_server")
      run_ossl_server
      ;;
  "ossl_client")
      run_ossl_client
      ;;
  "loopback")
      run_loopback
      ;;
  *)
    echo "command not found"
    ;;
esac
