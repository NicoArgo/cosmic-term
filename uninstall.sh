#!/usr/bin/env bash
# Restore the original system cosmic-term (undo install.sh).
set -euo pipefail
cd "$(dirname "$0")"

[ -f cosmic-term.orig ] || { echo "No backup (cosmic-term.orig) found."; exit 1; }

# The backup was taken whenever this fork was first installed, and apt has moved
# since on this machine before. Restoring a backup older than the package would
# be a silent downgrade — undoing more than this script promises to undo.
BACKUP_VER="$(./cosmic-term.orig --version 2>/dev/null | awk '{print $2}')"
PKG_VER="$(dpkg-query -W -f='${Version}' cosmic-term 2>/dev/null | cut -d'~' -f1)"
if [ -n "$BACKUP_VER" ] && [ -n "$PKG_VER" ] && [ "$BACKUP_VER" != "$PKG_VER" ]; then
    echo
    echo "!! The backup is cosmic-term $BACKUP_VER, but the package installed"
    echo "   on this system is $PKG_VER."
    echo
    echo "   Restoring the backup would put $BACKUP_VER back — a downgrade, and"
    echo "   not what 'undo the fork' should mean."
    echo
    echo "   Let apt restore its own binary instead, which is the real original:"
    echo "       sudo apt install --reinstall cosmic-term"
    echo "       rm $(pwd)/cosmic-term.orig"
    echo
    echo "   To restore the backup anyway, knowing the above:"
    echo "       POP_FLOW_ALLOW_STALE_BACKUP=1 ./uninstall.sh"
    echo
    [ "${POP_FLOW_ALLOW_STALE_BACKUP:-0}" = "1" ] || exit 1
fi

# Turn off auto-reapply first, or the next package operation would re-patch the
# binary right after we restore the original. Delegating to the script that owns
# those paths rather than repeating them: this used to remove only the golden
# copy, leaving a root-owned APT hook behind for good.
if [ -x ./remove-auto-reapply.sh ]; then
    ./remove-auto-reapply.sh
fi

echo "==> Restoring original /usr/bin/cosmic-term (needs sudo)..."
sudo install -m 0755 cosmic-term.orig /usr/bin/cosmic-term

echo "==> Restored. Open a new terminal window to load the stock binary."
echo "    Your dir_rules config is left in place and simply goes unused."
