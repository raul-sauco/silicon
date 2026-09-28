#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS] [DEVICE] [LOG]

Log LoRa water meter data from serial port to CSV file.

Arguments:
  DEVICE    Serial device to read from (default: /dev/ttyUSB0)
  LOG       Log file to append to     (default: flow_meter.log)

Options:
  -h        Show this help message

Output format:
  unix_timestamp,node_id,packet_id,edge_count,battery_mv,soc,flags,rssi,snr

Examples:
  $(basename "$0")
  $(basename "$0") /dev/ttyUSB1
  $(basename "$0") /dev/ttyUSB0 custom.log
EOF
}

while getopts "h" opt; do
    case $opt in
        h) usage; exit 0 ;;
        *) usage; exit 1 ;;
    esac
done
shift $((OPTIND - 1))

DEVICE="${1:-/dev/ttyUSB0}"
LOG="${2:-flow_meter.log}"

echo "Logging $DEVICE → $LOG"
# picocom -b 9600 "$DEVICE" | rts '%.s,' | tee -a "$LOG"

picocom -b 9600 "$DEVICE" |
while IFS= read -r line; do
    printf '%s,%s\n' "$(date +%s)" "$line"
done | tee -a "$LOG"
