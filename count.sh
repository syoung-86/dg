#!/bin/bash

# Get the current date
current_date=$(date +"%Y-%m-%d")

# Execute the command and capture the last line
last_line=$(find lib/ client/ server/ -name '*.rs' | xargs wc -l | tail -n 1 | awk '{print $1}')

# Output the current date and the first word of the last line
echo "$current_date $last_line" >> lines.txt
