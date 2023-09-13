#!/bin/bash

set -e

bun install

cd clients/web

bun run test:unit

bun run test:api
