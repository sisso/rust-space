#!/usr/bin/env bash

$(cd rust && cargo build)
python python/test1.py
