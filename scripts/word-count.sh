#!/bin/bash

word_to_count=$1
directory=$2

if [ -z "$word_to_count" ] || [ -z "$directory" ]; then
  echo "Usage: $0 word directory"
  exit 1
fi

count=0
for file in "$directory"/*.rs; do
  count_in_file=$(grep -o "$word_to_count" "$file" | wc -l)
  count=$((count + count_in_file))
done

echo "The word '$word_to_count' was found $count times in files with '.rs' extension in the '$directory' directory."