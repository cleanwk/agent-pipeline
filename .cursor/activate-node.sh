# Activate the Node.js 24 toolchain for Agent Pipeline.
#
# The Cloud Agent base image ships an older Node under /exec-daemon that sits at
# the front of PATH, so simply relying on `nvm use` is not enough: we must
# prepend the nvm-managed Node 24 bin directory explicitly so it wins.
#
# Meant to be sourced (`. .cursor/activate-node.sh`), not executed.

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
if [ -s "$NVM_DIR/nvm.sh" ]; then
  # shellcheck disable=SC1091
  . "$NVM_DIR/nvm.sh" >/dev/null 2>&1
fi

_ap_node24_bin="$(ls -d "$NVM_DIR"/versions/node/v24*/bin 2>/dev/null | sort -V | tail -1)"
if [ -n "$_ap_node24_bin" ]; then
  export PATH="$_ap_node24_bin:$PATH"
fi
unset _ap_node24_bin
hash -r 2>/dev/null || true
