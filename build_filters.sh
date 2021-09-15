#!/bin/sh

cd target && mkdir filters_build && cd filters_build
cmake -DCMAKE_BUILD_TYPE="$1" ../../filters
cmake --build .
