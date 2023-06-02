#!/usr/bin/env bash

## This script is based on the https://github.com/Cardinal-Cryptography/zk-apps/blob/main/shielder/deploy/deploy.sh

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

export NODE_IMAGE="public.ecr.aws/p6e8q1z1/aleph-node-liminal:d93048e"
export INK_DEV_IMAGE="public.ecr.aws/p6e8q1z1/ink-dev:1.1.0"

# actors
DAMIAN=//0
DAMIAN_PUBKEY=5D34dL5prEUaGNQtPPZ3yN5Y6BnkfXunKXXz6fo7ZJbLwRRH
HANS=//1
HANS_PUBKEY=5GBNeWRhZc2jXu7D55rBimKYDk8PGk8itRYFTPfC8RJLKG5o
ADMIN=//Alice
ADMIN_PUBKEY=5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY

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
        -u "${DOCKER_USER}" \
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

random_salt() {
  hexdump -vn16 -e'4/4 "%08X" 1 "\n"' /dev/urandom
}

contract_instantiate() {
  docker_ink_dev "cargo contract instantiate --skip-confirm --url ${NODE} --suri ${ADMIN} --output-json --salt 0x$(random_salt) ${1}"
}

deploy_contract() {
    cd "${SCRIPT_DIR}"/..
    CONTRACT_ADDRESS=$(contract_instantiate "--args true --manifest-path contract/Cargo.toml" | jq -r '.contract')
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

# ------------------------------------------------------------------------------------------------------

deploy() {
    # general setup
    prepare_fs

    # launching node
    generate_chainspec
    export_bootnode_address
    run_snarkeling_node

    # build contracts
    build

    # deploy
    deploy_contract

    # store data
    store_contract_addres

    log_progress "🙌 Deployment successful"
}

deploy

set +euo pipefail
