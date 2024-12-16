#!/bin/bash
#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#

# Check if the first argument is provided
if [ -z "$1" ]; then
  echo "Usage: $0 /path/to/folder"
  exit 1
fi

# Use the first command-line argument
FOLDER="$1"
MAX_USAGE=0

while true; do
  # Get current usage in KB
  CURRENT_USAGE_KB=$(du -s "$FOLDER" | awk '{print $1}')

  # Convert KB to GB with floating point precision
  CURRENT_USAGE_GB=$(echo "scale=2; $CURRENT_USAGE_KB / 1024 / 1024" | bc)

  # Compare and update the max usage
  if (($(echo "$CURRENT_USAGE_GB > $MAX_USAGE" | bc -l))); then
    MAX_USAGE=$CURRENT_USAGE_GB
  fi

  # Print the current and max usage
  echo "Current: ${CURRENT_USAGE_GB} GB, Max: ${MAX_USAGE} GB"
  sleep 1
done
