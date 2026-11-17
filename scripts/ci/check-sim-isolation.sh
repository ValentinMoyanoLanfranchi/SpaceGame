#!/usr/bin/env bash
# Verifica que `heliarca-sim` no dependa de Godot (principio de arquitectura, doc 21 §2):
# ningún crate de su árbol de dependencias (normal, build y dev; todas las features y
# targets) puede tener un nombre que contenga `godot`, `gdext` o `gdnative`.
# Usa --locked para que el chequeo no reescriba Cargo.lock y oculte un lockfile desactualizado.
# Uso: scripts/ci/check-sim-isolation.sh [raiz-del-repo]   (por defecto, el directorio actual)
set -euo pipefail

raiz="${1:-.}"

fallar() { echo "ERROR: $*" >&2; exit 1; }

[ -f "$raiz/Cargo.toml" ] || fallar "no existe Cargo.toml en '$raiz'."

# Se captura la salida primero: así un fallo de cargo no puede pasar por "sin coincidencias".
if ! arbol="$(cd "$raiz" && cargo tree --locked -p heliarca-sim --edges normal,build,dev \
    --all-features --target all --prefix none 2>&1)"; then
  echo "$arbol" >&2
  fallar "'cargo tree -p heliarca-sim' falló (¿Cargo.lock desactualizado?); no se pudo comprobar el aislamiento."
fi

# Primera columna de cada línea = nombre del crate.
prohibidos="$(awk '{print $1}' <<<"$arbol" | grep -iE 'godot|gdext|gdnative' | sort -u || true)"

if [ -n "$prohibidos" ]; then
  echo "Árbol de dependencias de heliarca-sim:" >&2
  echo "$arbol" >&2
  echo >&2
  echo "Crates prohibidos encontrados:" >&2
  echo "$prohibidos" >&2
  fallar "heliarca-sim depende de Godot (doc 21 §2): la simulación debe ser independiente del motor."
fi

echo "OK: heliarca-sim no depende de Godot (gdext/godot/gdnative)."
