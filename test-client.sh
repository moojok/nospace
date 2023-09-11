#!/bin/bash

set -e

cd clients/web

yarn install

yarn test:unit

yarn test:api
