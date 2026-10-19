#!/bin/bash
cargo clean && \
cargo +nightly build --release --target armv4t-none-eabi -Zbuild-std=core,alloc "$@" && \
arm-none-eabi-objcopy -O binary target/armv4t-none-eabi/release/ppdeluxe linux.bin
