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

canal="$(sed -nE 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"([^"]*)".*/\1/p' "$toolchain" | head -n 1)"
[ -n "$canal" ] || fallar "rust-toolchain.toml no define 'channel'."

if ! [[ "$canal" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  fallar "channel = \"$canal\" no es una versión exacta X.Y.Z (no se permite stable, beta, nightly ni versiones parciales)."
fi

[ -f "$readme" ] || fallar "no existe README.md."

if ! grep -qF -- "$canal" "$readme"; then
  fallar "README.md no menciona la versión $canal fijada en rust-toolchain.toml; actualizá la documentación."
fi

echo "OK: toolchain fijado en $canal y documentado en README.md."
