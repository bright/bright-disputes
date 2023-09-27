#!/usr/bin/env bash

## This script is based on the https://github.com/Cardinal-Cryptography/zk-apps/blob/main/shielder/deploy/deploy.sh

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

export NODE_IMAGE="brightinventions/disputes-node"
export CLIAIN_IMAGE="brightinventions/disputes-cliain"
export INK_DEV_IMAGE="brightinventions/disputes-ink-dev"

# actors
DAMIAN=//0
DAMIAN_PUBKEY=5D34dL5prEUaGNQtPPZ3yN5Y6BnkfXunKXXz6fo7ZJbLwRRH
HANS=//1
HANS_PUBKEY=5GBNeWRhZc2jXu7D55rBimKYDk8PGk8itRYFTPfC8RJLKG5o
MICHAL=//2
MICHAL_PUBKEY=5H8rhTXiLiXAe9yhnnQrCuz6bvbwrcTddMJa9KfsX9mi26sj
ADMIN=//Alice
ADMIN_PUBKEY=5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY

OWNER=//Owner
OWNER_PUBKEY=5FTyuyEQQZs8tCcPTUFqotkm2SYfDnpefn9FitRgmTHnFDBD
DEFENDANT=//Defendant
DEFENDANT_PUBKEY=5HpJbr84AqocNWyq4WNAQLNLSNNoXVmqAhvrk8Tq7YX23j6p
JUROR_1=//Juror1
JUROR_1_PUBKEY=5GvG1edSDSrAG5HZ21N1BVGEgygpSujAAjuruyfyuCgsgEFr
JUROR_2=//Juror2
JUROR_2_PUBKEY=5DZyhVcMqnfg78WK8EsUyu3tpLb2peARqVEoieEunsgH2iQb
JUROR_3=//Juror3
JUROR_3_PUBKEY=5FjEKpjdvNe8SbZjgUaF8qQD3mL9T5k8oCtGozMYkHi2aVCi
JUROR_4=//Juror4
JUROR_4_PUBKEY=5H4SHcV6XVFiGF3QGdKFLES3xwUmN9jFdt5KVNapJtfWPPtT
JUROR_5=//Juror5
JUROR_5_PUBKEY=5G492oT3GwqTpz4ebV15JHERucL96zEp54TZZSm3ZQHGe9AE
JUROR_6=//Juror6
JUROR_6_PUBKEY=5GjNM6gLeYxeB9aQoPxVVa7H494ijFsHTXTNo9dkNuTyDCeD
JUROR_7=//Juror7
JUROR_7_PUBKEY=5F71WWxPWDKRJt5x4LudQ3k94GCC8WATnRm86FWmvdEYJpnB

# tokenomics
TOKEN_PER_PERSON=1000

# env
NODE="ws://127.0.0.1:9944"
DOCKER_USER="$(id -u):$(id -g)"
export DOCKER_USER

# command aliases
DOCKER_SH="docker run --rm -e RUST_LOG=debug -u ${DOCKER_USER} --entrypoint /bin/sh"

get_timestamp() {
    date +'%Y-%m-%d %H:%M:%S'
}

log_progress() {
    bold=$(tput bold)
    normal=$(tput sgr0)
    echo "[$(get_timestamp)] [INFO] ${bold}${1}${normal}"
}

prepare_fs() {
    # ensure that we are in scripts folder
    cd "${SCRIPT_DIR}"

    # forget everything from the past launches - start the chain from a scratch
    rm -rf docker/node_data/

    # ensure that all these folders are present
    mkdir -p docker/node_data/
    mkdir -p docker/keys/

    log_progress "✅ Directories are set up"
}

generate_chainspec() {
    CHAINSPEC_ARGS="\
    --base-path /data \
    --account-ids ${DAMIAN_PUBKEY} \
    --sudo-account-id ${ADMIN_PUBKEY} \
    --faucet-account-id ${ADMIN_PUBKEY} \
    --chain-id a0smnet \
    --token-symbol SNZERO \
    --chain-name 'Aleph Zero Snarkeling'"

    $DOCKER_SH \
        -v "${SCRIPT_DIR}/docker/node_data:/data" \
        "${NODE_IMAGE}" \
        -c "aleph-node bootstrap-chain ${CHAINSPEC_ARGS} > /data/chainspec.snarkeling.json"

    log_progress "✅ Generated chainspec was written to docker/data/chainspec.snarkeling.json"
}

export_bootnode_address() {
    BOOTNODE_PEER_ID=$(
        $DOCKER_SH \
            -v "${SCRIPT_DIR}/docker/node_data:/data" \
            "${NODE_IMAGE}" \
            -c "aleph-node key inspect-node-key --file /data/${DAMIAN_PUBKEY}/p2p_secret"
    )
    export BOOTNODE_PEER_ID
    log_progress "✅ Exported bootnode address (${BOOTNODE_PEER_ID})"
}

run_snarkeling_node() {
    NODE_PUBKEY=$DAMIAN_PUBKEY docker-compose -f docker-compose.yml up --remove-orphans -d
    log_progress "✅ Successfully launched snarkeling node"
}

docker_ink_dev() {
    docker run --rm \
        -v "${PWD}":/code \
        -v ~/.cargo/git:/usr/local/cargo/git \
        -v ~/.cargo/registry:/usr/local/cargo/registry \
        --network host \
        --entrypoint /bin/sh \
        "${INK_DEV_IMAGE}" \
        -c "${1}"
}

build() {
    cd "${SCRIPT_DIR}"/..

    docker_ink_dev "cargo contract build --release --manifest-path contract/Cargo.toml 1>/dev/null"
    log_progress "✅ Contract was built"
}

build_cli() {
  cd "${SCRIPT_DIR}"/..
  docker_ink_dev "cargo +nightly-2023-04-16 build --release --manifest-path cli/Cargo.toml 1>/dev/null"
  log_progress "✅ CLI was built"

  cd "${SCRIPT_DIR}"/../cli/
  docker_ink_dev "./target/release/bright_disputes_cli set-contract ${CONTRACT_ADDRESS} 1>/dev/null"

  log_progress "✅ Shielder CLI was set up"
}

random_salt() {
    hexdump -vn16 -e'4/4 "%08X" 1 "\n"' /dev/urandom
}

contract_instantiate() {
    docker_ink_dev "cargo contract instantiate --skip-confirm --url ${NODE} --suri ${ADMIN} --output-json --execute --salt 0x$(random_salt) ${1}"
}

deploy_contract() {
    cd "${SCRIPT_DIR}"/..
    CONTRACT_ADDRESS=$(contract_instantiate "--manifest-path contract/Cargo.toml" | jq -r '.contract')
    export CONTRACT_ADDRESS
    log_progress "Contract address: ${CONTRACT_ADDRESS}"
}

store_contract_addres() {
    jq -n --arg contract_address "$CONTRACT_ADDRESS" \
        '{
          contract_address: $contract_address,
        }' >${SCRIPT_DIR}/addresses.json

    log_progress "✅ Contract addresses stored in a ${SCRIPT_DIR}/addresses.json"
}

transfer() {
    $DOCKER_SH \
        --network host \
        ${CLIAIN_IMAGE} \
        -c "/usr/local/bin/cliain --node ${NODE} --seed ${ADMIN} transfer --amount-in-tokens ${TOKEN_PER_PERSON} --to-account ${1}" 1>/dev/null

    log_progress "✅ Transferred ${TOKEN_PER_PERSON} to ${1}"
}

prefund_users() {
    for recipient in "${DAMIAN_PUBKEY}" "${HANS_PUBKEY}" "${MICHAL_PUBKEY}" "${OWNER_PUBKEY}" "${DEFENDANT_PUBKEY}" "${JUROR_1_PUBKEY}" \
     "${JUROR_2_PUBKEY}" "${JUROR_3_PUBKEY}" "${JUROR_4_PUBKEY}" "${JUROR_5_PUBKEY}" "${JUROR_6_PUBKEY}" "${JUROR_7_PUBKEY}"; do
        transfer ${recipient}
    done
}

generate_ink_types() {
  docker_ink_dev "ink-wrapper -m contract/target/ink/bright_disputes.json | rustfmt +nightly-2023-04-16 --edition 2021 > cli/src/bright_disputes_ink.rs"

  log_progress "✅ Ink types were generated"
}

# ------------------------------------------------------------------------------------------------------

deploy() {
    # general setup
    prepare_fs

    # launching node
    generate_chainspec
    export_bootnode_address
    run_snarkeling_node

    # prefund users
    prefund_users

    # build contracts
    build

    # generate ink-wrapper types
    generate_ink_types

    # deploy
    deploy_contract

    # store data
    store_contract_addres

    # build cli
    build_cli

    log_progress "🙌 Deployment successful"
}

deploy

set +euo pipefail
