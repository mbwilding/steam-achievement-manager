#!/usr/bin/env bash

./patch.sh

cargo publish -p open_ecc
cargo publish -p open_ecc_cli
