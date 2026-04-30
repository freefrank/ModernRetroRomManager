#!/bin/bash
# Fix PSP metadata.pegasus.txt - Add folder paths to file: fields
# Usage: bash fix_psp_metadata.sh

PSP_DIR="/d/samba/psp"
METADATA_FILE="$PSP_DIR/metadata.pegasus.txt"
OUTPUT_FILE="$PSP_DIR/metadata.pegasus.fixed.txt"

echo "Processing $METADATA_FILE..."

# Process line by line
while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ "$line" =~ ^file:\ (.+)$ ]]; then
        filename="${BASH_REMATCH[1]}"
        
        # Check if already has a path separator
        if [[ "$filename" == *"/"* ]] || [[ "$filename" == *"\\"* ]]; then
            # Already has path, keep as is
            echo "$line"
        else
            # Find the file in subdirectories
            found_path=$(find "$PSP_DIR" -maxdepth 2 -type f -name "$filename" 2>/dev/null | head -1)
            
            if [[ -n "$found_path" ]]; then
                # Extract relative path (folder/filename)
                relative_path="${found_path#$PSP_DIR/}"
                echo "file: $relative_path"
            else
                # File not found, keep original
                echo "$line"
            fi
        fi
    else
        echo "$line"
    fi
done < "$METADATA_FILE" > "$OUTPUT_FILE"

echo "Done! Output saved to $OUTPUT_FILE"
echo "Please review and rename to metadata.pegasus.txt if correct."
