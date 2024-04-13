#!/usr/bin/env bash

echo $GITHUB_WORKSPACE && \
    ls -la $GITHUB_WORKSPACE && \
    git config --global --add safe.directory $GITHUB_WORKSPACE && \
    cd $GITHUB_WORKSPACE && ls -la . && \
    git status
