#!/bin/bash
# Script to generate PHYSICAL_PROOF.txt with content 'AGENCY_CONFIRMED'

# Ensure the directory exists
mkdir -p "$(dirname "$0")/."

# Write the content to the file
echo "AGENCY_CONFIRMED" > PHYSICAL_PROOF.txt

# Calculate and store SHA256 checksum
certutil -hashfile PHYSICAL_PROOF.txt SHA256 | findstr /v "hashfile: PHYSICAL_PROOF.txt" | findstr /v "CertUtil: -hashfile command completed successfully." > PHYSICAL_PROOF.txt.sha256


echo "PHYSICAL_PROOF.txt generated successfully."
