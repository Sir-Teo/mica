#!/bin/bash
set -e

echo "📝 Generating examples manifest..."

OUTPUT_FILE="docs/examples-manifest.json"

# Start JSON array
echo "[" > "$OUTPUT_FILE"

# Find all .mica files in examples directory
first=true
for file in examples/*.mica; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        name="${filename%.mica}"
        
        # Add comma for all but first entry
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "$OUTPUT_FILE"
        fi
        
        # Read the file content and escape it for JSON
        content=$(cat "$file" | jq -Rs .)
        
        # Get first line as description if it's a comment
        description=""
        first_line=$(head -n 1 "$file")
        if [[ $first_line == //* ]]; then
            description="${first_line#// }"
        fi
        
        # Write JSON entry
        cat >> "$OUTPUT_FILE" << EOF
  {
    "id": "$name",
    "name": "$filename",
    "description": "$description",
    "code": $content
  }
EOF
    fi
done

# Close JSON array
echo "" >> "$OUTPUT_FILE"
echo "]" >> "$OUTPUT_FILE"

echo "✅ Manifest generated: $OUTPUT_FILE"
echo "📊 Total examples: $(ls -1 examples/*.mica | wc -l | tr -d ' ')"
