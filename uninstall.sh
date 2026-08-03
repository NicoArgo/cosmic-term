#!/usr/bin/env bash
# Restore the original system cosmic-term (undo install.sh).
set -euo pipefail
cd "$(dirname "$0")"

[ -f cosmic-term.orig ] || { echo "No backup (cosmic-term.orig) found."; exit 1; }

# Remove the auto-reapply golden copy if one was placed for cosmic-term.
if [ -f /usr/local/lib/pop-flow/cosmic-term ]; then
    echo "==> Removing auto-reapply golden copy (needs sudo)..."
    sudo rm -f /usr/local/lib/pop-flow/cosmic-term
fi

echo "==> Restoring original /usr/bin/cosmic-term (needs sudo)..."
sudo install -m 0755 cosmic-term.orig /usr/bin/cosmic-term

echo "==> Restored. Open a new terminal window to load the stock binary."
echo "    Your dir_rules config is left in place and simply goes unused."
