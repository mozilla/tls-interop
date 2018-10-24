#!/usr/bin/env bash
BASE_DIR="$(cd "$(dirname "$0")"; pwd -P)"
CASE_FILE="cases.json"
MODE=""

print_help() {
  printf "Usage: -m <mode> [-c <case_file>] [-v]\n\n \
    -v      Verbose output.\n \
    -m      test mode. ( all | loopback | ossl_server | ossl_client | bssl_server | bssl_shim ) \n \
    -c      case file. Default is: cases.json\n"
}

while [ $# -gt 0 ]; do
    case "$1" in
        -v) export RUST_LOG=debug ;;
        -m) MODE="$2"; shift ;;
        -c) CASE_FILE="$2"; shift;;
        *) print_help; exit 2 ;;
    esac
    shift
done

CERT_DIR="$BASE_DIR/../boringssl/ssl/test/runner/"
declare -A shims=([nss]="$BASE_DIR/../dist/Debug/bin/nss_bogo_shim" \
                  [bssl]="$BASE_DIR/../boringssl/build/ssl/test/bssl_shim" \
                  [ossl]="$BASE_DIR/../openssl/test/ossl_shim/ossl_shim")

run_shim_pair() {
  SHIM1="$1"
  SHIM2="$2"

  args=()

  # If NSS acts as the client, interop needs this argument.
  # It would become obsolete if bssl and ossl could actively initiate
  # communication after the handshake.
  if [[ $SHIM1 = "nss" ]]; then
    args+=(--client-writes-first)
  fi

  # The ossl_shim is currently not properly IPv6 capable, which is why interop
  # needs this argument when ossl_shim is involved in the test case.
  if [[ $SHIM1 == "ossl" ]] || [[ $SHIM2 == "ossl" ]] ; then
    args+=(--force-IPv4)
  fi

  cargo run -- \
  --client "${shims[$SHIM1]}" \
  --server "${shims[$SHIM2]}" \
  --rootdir "$CERT_DIR" \
  --test-cases "$BASE_DIR/$CASE_FILE" \
  ${args[@]}
}

run_mode() {
  if [[ "$1" == "$2" ]] || [[ "$1" == "all" ]]; then
    run_shim_pair "$3" "$4"
    invalid_mode=
  fi
}

invalid_mode=true
run_mode "$MODE" bssl_server "nss" "bssl"
run_mode "$MODE" bssl_client "bssl" "nss"
run_mode "$MODE" ossl_server "nss" "ossl"
run_mode "$MODE" ossl_client "ossl" "nss"
run_mode "$MODE" loopback "nss" "nss"

[[ -z "$invalid_mode" ]] || print_help
