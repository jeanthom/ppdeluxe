#!/bin/bash
cargo clean && cargo +nightly build --release --target armv4t-none-eabi -Zbuild-std && arm-none-eabi-objcopy -O binary target/armv4t-none-eabi/release/scythe linux.bin