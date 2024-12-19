#!/bin/bash

# Copy the pre-commit hook to git hooks directory
cp ../../../../.githooks/pre-commit ../../../../.git/hooks/pre-commit

# Make the hook executable
chmod +x ../../../../.git/hooks/pre-commit

echo "Git pre-commit hook installed successfully"
