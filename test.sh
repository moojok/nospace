#!/bin/bash

set -e

cargo test --workspace --lib -- --nocapture
