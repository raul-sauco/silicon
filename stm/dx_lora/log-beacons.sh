#!/usr/bin/env bash

DEVICE="${1:-/dev/ttyUSB0}"
LOG="${2:-beacons.log}"

picocom -b 9600 "$DEVICE" |
while IFS= read -r line; do
    printf '%s %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%S.%3NZ')" "$line"
done | tee -a "$LOG"
