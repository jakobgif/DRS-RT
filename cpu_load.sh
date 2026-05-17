#!/bin/bash
# TF-1: CPU load script for DRS-RT project

CORES=$(nproc 2>/dev/null || echo 4)
echo "Starting busy-loop workers on $CORES cores..."

trap "kill 0" SIGINT SIGTERM

for i in $(seq 1 $CORES); do
    yes > /dev/null &
done

wait
