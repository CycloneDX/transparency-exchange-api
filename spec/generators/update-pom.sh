#!/bin/bash

# Check if the file path is provided
if [ -z "$1" ]; then
  echo "Usage: $0 <path-to-xml-file>"
  exit 1
fi

# File path
FILE=$1

# Use sed to remove the <build> tag and its content
sed -i.bak '/<build>/,/<\/build>/d' "$FILE"

echo "The <build> tag has been removed from $FILE. A backup has been saved as $FILE.bak."