#!/usr/bin/env bash
# Build and install POP Flow's cosmic-term (per-directory appearance) over the
# system cosmic-term. Reversible with ./uninstall.sh
set -euo pipefail
cd "$(dirname "$0")"

echo "==> Building (cargo build --release)..."
cargo build --release

BIN="target/release/cosmic-term"
[ -f "$BIN" ] || { echo "Build failed: $BIN not found"; exit 1; }

# The backup taken here is what ./uninstall.sh restores, so it is worth one
# check that we are not about to freeze an outdated binary as "the original".
# This machine sat a whole epoch behind upstream while this fork was made.
if [ ! -f cosmic-term.orig ] && [ -f /usr/bin/cosmic-term ]; then
    SYSTEM_VER="$(/usr/bin/cosmic-term --version 2>/dev/null | awk '{print $2}')"
    OURS_VER="$(awk -F'"' '/^version = /{print $2; exit}' Cargo.toml)"
    if [ -n "$SYSTEM_VER" ] && [ -n "$OURS_VER" ] && [ "$SYSTEM_VER" != "$OURS_VER" ]; then
        echo
        echo "!! The installed cosmic-term is $SYSTEM_VER, but this fork is based"
        echo "   on upstream $OURS_VER."
        echo
        echo "   The backup about to be taken is what ./uninstall.sh restores, so"
        echo "   taking it now would freeze $SYSTEM_VER as 'the original' and"
        echo "   uninstalling later would quietly downgrade you to it."
        echo
        echo "   Bring the package up to date first, then run this again:"
        echo "       sudo apt install --only-upgrade cosmic-term"
        echo
        echo "   (--reinstall would put $SYSTEM_VER back, not the current release.)"
        echo
        echo "   To take the backup anyway, knowing the above:"
        echo "       POP_FLOW_ALLOW_STALE_BACKUP=1 ./install.sh"
        # Refuses rather than warns. This used to print a warning and carry on,
        # and a warning in the middle of a long build scrolls past — the backup
        # was already wrong by the time anyone read it.
        [ "${POP_FLOW_ALLOW_STALE_BACKUP:-0}" = "1" ] || exit 1
    fi
fi

if [ ! -f cosmic-term.orig ] && [ -f /usr/bin/cosmic-term ]; then
    echo "==> Backing up current /usr/bin/cosmic-term -> ./cosmic-term.orig"
    cp /usr/bin/cosmic-term cosmic-term.orig
fi

echo "==> Installing to /usr/bin/cosmic-term (needs sudo)..."
sudo install -m 0755 "$BIN" /usr/bin/cosmic-term

# Keep the auto-reapply golden copy in sync, or say out loud that this install
# is temporary — silence here used to hide the fact that a package update wipes
# the feature.
GOLDEN=/usr/local/lib/pop-flow/cosmic-term
if [ -f "$GOLDEN" ]; then
    echo "==> Refreshing auto-reapply golden copy"
    sudo install -m 0755 "$BIN" "$GOLDEN"
else
    echo "!! No auto-reapply hook installed: the next package update of"
    echo "   cosmic-term will silently restore the stock binary."
    echo "   Run ./setup-auto-reapply.sh to make this install stick."
fi

echo "==> Done."
echo "    Note: terminals already open keep the OLD binary — including this one."
echo "    Open a NEW terminal window to get per-directory appearance."
echo "    (No process is killed here: doing so would close your open shells.)"
echo
echo "    To use it: right-click in a terminal -> 'Use this appearance here',"
echo "    then File -> 'Directory rules...' to adjust. See README.md."
