#!/usr/bin/env bash
# Verifica que el pin del toolchain de Rust no se rompa:
#   (a) existe rust-toolchain.toml
#   (b) su `channel` es una versión exacta X.Y.Z (no stable/beta/nightly)
#   (c) README.md menciona esa misma versión
# Uso: scripts/ci/check-toolchain.sh [raiz-del-repo]   (por defecto, el directorio actual)
set -euo pipefail

raiz="${1:-.}"
toolchain="$raiz/rust-toolchain.toml"
readme="$raiz/README.md"

fallar() { echo "ERROR: $*" >&2; exit 1; }

[ -f "$toolchain" ] || fallar "no existe rust-toolchain.toml en la raíz del repo; el toolchain debe estar fijado."

# Busca `channel` solo dentro de la tabla [toolchain]; acepta comillas dobles o simples.
canal="$(awk -v q="'" '
  /^[[:space:]]*\[/ { en = ($0 ~ /^[[:space:]]*\[toolchain\][[:space:]]*(#.*)?$/); next }
  en && /^[[:space:]]*channel[[:space:]]*=/ {
    v = $0; sub(/^[^=]*=[[:space:]]*/, "", v)
    if (match(v, /^"[^"]*"/) || match(v, "^" q "[^" q "]*" q)) { print substr(v, 2, RLENGTH - 2); exit }
  }' "$toolchain")"
[ -n "$canal" ] || fallar "rust-toolchain.toml no define 'channel' en la tabla [toolchain]."

if ! [[ "$canal" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  fallar "channel = \"$canal\" no es una versión exacta X.Y.Z (no se permite stable, beta, nightly ni versiones parciales)."
fi

[ -f "$readme" ] || fallar "no existe README.md."

# Coincidencia de versión completa: puntos escapados y sin dígito (ni ".<dígito>") pegado
# antes o después, pero se tolera un punto final de oración ("...1.100.0.").
canal_re="${canal//./\.}"
if ! grep -qE -- "(^|[^0-9.]|[^0-9]\.)${canal_re}(\$|[^0-9.]|\.(\$|[^0-9]))" "$readme"; then
  fallar "README.md no menciona la versión $canal fijada en rust-toolchain.toml; actualizá la documentación."
fi

echo "OK: toolchain fijado en $canal y documentado en README.md."
