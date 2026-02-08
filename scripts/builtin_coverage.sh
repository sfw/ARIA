#!/usr/bin/env bash
# builtin_coverage.sh — Audit builtin function test coverage
#
# Extracts all builtin names from the interpreter match arms and checks
# which ones have corresponding tests.
#
# Usage: bash scripts/builtin_coverage.sh
# Exit: always 0 (informational)

set -euo pipefail

INTERP="src/mir/interp.rs"

if [ ! -f "$INTERP" ]; then
    echo "Error: $INTERP not found. Run from the project root."
    exit 0
fi

# Extract builtin names from match arms: patterns like "builtin_name" =>
# We use sed to extract quoted strings followed by =>
BUILTINS=$(grep -E '^\s+"[a-z_]+" *=>' "$INTERP" | sed 's/.*"\([a-z_]*\)".*/\1/' | sort -u)

TOTAL=$(echo "$BUILTINS" | wc -l | tr -d ' ')

# Extract tested builtins from test functions:
# 1. call_builtin("name", ...) patterns
TESTED_CALL=$(grep -o 'call_builtin("[a-z_]*"' "$INTERP" | sed 's/call_builtin("//;s/"//' | sort -u)

# Also check test files in tests/ directory
TESTED_TESTS=""
if [ -d "tests" ]; then
    TESTED_TESTS=$(grep -ro 'call_builtin("[a-z_]*"' tests/ 2>/dev/null | sed 's/.*call_builtin("//;s/"//' | sort -u || true)
fi

# Combine all tested builtins
TESTED=$(printf '%s\n%s\n' "$TESTED_CALL" "$TESTED_TESTS" | sort -u | grep -v '^$' || true)
TESTED_COUNT=$(echo "$TESTED" | grep -c . || echo 0)

# Find untested builtins
UNTESTED=""
UNTESTED_COUNT=0
while IFS= read -r builtin; do
    if ! echo "$TESTED" | grep -qx "$builtin"; then
        UNTESTED="${UNTESTED}  ${builtin}
"
        UNTESTED_COUNT=$((UNTESTED_COUNT + 1))
    fi
done <<< "$BUILTINS"

# Calculate coverage percentage
if [ "$TOTAL" -gt 0 ]; then
    COVERED=$((TOTAL - UNTESTED_COUNT))
    PCT=$((COVERED * 100 / TOTAL))
else
    COVERED=0
    PCT=0
fi

echo "========================================="
echo "  FORMA Builtin Coverage Report"
echo "========================================="
echo ""
echo "Total builtins:   $TOTAL"
echo "Tested builtins:  $COVERED"
echo "Untested:         $UNTESTED_COUNT"
echo "Coverage:         ${PCT}%"
echo ""

if [ "$UNTESTED_COUNT" -gt 0 ]; then
    echo "Untested builtins:"
    echo "$UNTESTED"
fi

echo "========================================="

exit 0
