#!/usr/bin/env bash
BASE_DIR=$(cd $(dirname $0); pwd -P)
CASE_FILE="cases.json"
MODE=""

print_help() {
  printf "Help: \n \
        -v) Verbose output.\n \
        -m) Test mode. [ all | loopback | ossl_server | ossl_client | bssl_server | bssl_shim ]\n \
        -c) Case file. (Optional. Default cases.json) \n"
}

while [ $# -gt 0 ]; do
    case $1 in
        -v) export RUST_LOG=debug ;;
        -m) MODE="$2"; shift ;;
        -c) CASE_FILE="$2"; shift;;
        *) print_help; exit 2 ;;
    esac
    shift
done

CERT_DIR="$BASE_DIR/../boringssl/ssl/test/runner/"
NSS_SHIM="$BASE_DIR/../dist/Debug/bin/nss_bogo_shim"
BSSL_SHIM="$BASE_DIR/../boringssl/build/ssl/test/bssl_shim"
OSSL_SHIM="$BASE_DIR/../openssl/test/ossl_shim/ossl_shim"

SHIM_ARRAY=($NSS_SHIM $BSSL_SHIM $OSSL_SHIM)

run_shim_pair() {
  SHIM1=$1
  SHIM2=$2
  # If NSS acts as the client, interop needs this argument.
  # It would become obsolete if bssl and ossl could actively initiate
  # communication after the handshake.
  CLIENT_WRITES=""
  if [[ $SHIM1 = *"nss_bogo_shim"* ]]; then
    CLIENT_WRITES="--client-writes-first"
  fi

  # The ossl_shim is currently not properly IPv6 capable, which is why interop
  # needs this argument when ossl_shim is involved in the test case.
  IP4=""
  if [[ $SHIM1 = *"ossl"* ]] || [[ $SHIM2 = *"ossl"* ]] ; then
    IP4="--force-IPv4"
  fi

  cargo run -- \
  --client $SHIM1 \
  --server $SHIM2 \
  --rootdir $CERT_DIR \
  --test-cases $BASE_DIR/$CASE_FILE \
  $CLIENT_WRITES \
  $IP4
}

case $MODE in
  "all")
      for i in ${SHIM_ARRAY[@]}
        do
          for j in ${SHIM_ARRAY[@]}
          do
            # Currently at least one nss shim needs to be involved in a test
            # case because neither bssl_shim nor ossl_shim can actively
            # initiate the communication after a successful handshake.
            if [[ $i = *"nss_bogo_shim"* ]] || [[ $j = *"nss_bogo_shim"* ]] ; then
              run_shim_pair $i $j
            fi
          done
        done
      ;;

  # Hardcoded cases for all currently working combinations of shims are kept
  # for conveniently running certain shim pairs against each other during
  # development.
  "boring_server")
      run_shim_pair $NSS_SHIM $BSSL_SHIM
      ;;
  "boring_client")
      run_shim_pair $BSSL_SHIM $NSS_SHIM
      ;;
  "ossl_server")
      run_shim_pair $NSS_SHIM $OSSL_SHIM
      ;;
  "ossl_client")
      run_shim_pair $OSSL_SHIM $NSS_SHIM
      ;;
  "loopback")
      run_shim_pair $NSS_SHIM $NSS_SHIM
      ;;
  *)
    print_help
    ;;
esac
