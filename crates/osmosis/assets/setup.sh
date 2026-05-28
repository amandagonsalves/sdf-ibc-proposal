#!/bin/sh
set -e

CONFIG_JSON="${OSMOSIS_CONFIG_JSON:-/config/default-config.json}"
OSMOSIS_HOME=$HOME/.osmosisd
CONFIG_FOLDER=$OSMOSIS_HOME/config
TAB="$(printf '\t')"

apply_overrides () {
    target_file="$1"
    selector="$2"
    jq -r "${selector}[] | [.path, .type, (.value | tostring)] | @tsv" "$CONFIG_JSON" |
    while IFS="$TAB" read -r path type value; do
        dasel put -t "$type" -f "$target_file" -v "$value" "$path"
    done
}

add_genesis_accounts () {
    jq -r '.genesis_accounts[] | [.address, .coins] | @tsv' "$CONFIG_JSON" |
    while IFS="$TAB" read -r address coins; do
        osmosisd add-genesis-account "$address" "$coins" --home "$OSMOSIS_HOME"
    done
}

init_chain () {
    apk add --no-cache jq dasel

    CHAIN_ID="${OSMOSIS_CHAIN_ID:-$(jq -r '.chain_id' "$CONFIG_JSON")}"
    MONIKER="$(jq -r '.moniker' "$CONFIG_JSON")"
    GENESIS_TIME="${OSMOSIS_LOCAL_GENESIS_TIME:-$(jq -r '.genesis_time' "$CONFIG_JSON")}"
    VAL_MNEMONIC="$(jq -r '.keys.val' "$CONFIG_JSON")"
    POOLS_MNEMONIC="$(jq -r '.keys.pools' "$CONFIG_JSON")"
    GENTX_KEY="$(jq -r '.gentx.key' "$CONFIG_JSON")"
    GENTX_AMOUNT="$(jq -r '.gentx.amount' "$CONFIG_JSON")"

    echo "$VAL_MNEMONIC" | osmosisd init -o --chain-id="$CHAIN_ID" --home "$OSMOSIS_HOME" --recover "$MONIKER"

    apply_overrides "$CONFIG_FOLDER/genesis.json" '.genesis'
    dasel put -t string -f "$CONFIG_FOLDER/genesis.json" -v "$GENESIS_TIME" '.genesis_time'

    add_genesis_accounts
    echo "$VAL_MNEMONIC" | osmosisd keys add "$MONIKER" --recover --keyring-backend=test --home "$OSMOSIS_HOME"
    echo "$POOLS_MNEMONIC" | osmosisd keys add pools --recover --keyring-backend=test --home "$OSMOSIS_HOME"
    osmosisd gentx "$GENTX_KEY" "$GENTX_AMOUNT" --keyring-backend=test --chain-id="$CHAIN_ID" --home "$OSMOSIS_HOME"
    osmosisd collect-gentxs --home "$OSMOSIS_HOME"

    apply_overrides "$CONFIG_FOLDER/app.toml" '.app'
    apply_overrides "$CONFIG_FOLDER/config.toml" '.config'
}

if [ ! -d "$CONFIG_FOLDER" ]; then
    init_chain
fi

exec osmosisd start --home "$OSMOSIS_HOME"
