#!/bin/bash
set -ex

cargo watch -x test -i test_env --clear
