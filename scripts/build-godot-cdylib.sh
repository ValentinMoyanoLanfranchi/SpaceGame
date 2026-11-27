#!/usr/bin/env bash
# Compila el `cdylib` de `heliarca-godot` (sin Godot instalado) y verifica que haya quedado la
# biblioteca que `godot/heliarca.gdextension` declara para este sistema operativo (#61).
# `cargo build --workspace` ya lo compila de forma implícita, pero no notaría que el crate
# salió del workspace, dejó de ser `cdylib` o cambió de nombre.
# La biblioteca se borra antes de compilar: `target/` se cachea en el CI y una copia vieja
# haría pasar el chequeo.
# No está en `scripts/ci/` porque no es un chequeo de texto: el CI lo corre en Linux Y Windows.
# Uso: scripts/build-godot-cdylib.sh [raiz-del-repo]   (por defecto, el directorio actual)
set -euo pipefail

raiz="${1:-.}"

fallar() { echo "ERROR: $*" >&2; exit 1; }

extension="$raiz/godot/heliarca.gdextension"
[ -f "$raiz/Cargo.toml" ] || fallar "no existe Cargo.toml en '$raiz'."
[ -f "$extension" ] || fallar "no existe godot/heliarca.gdextension en '$raiz'."

case "$(uname -s)" in
  Linux) clave="linux.debug.x86_64" ;;
  MINGW* | MSYS* | CYGWIN*) clave="windows.debug.x86_64" ;;
  Darwin) clave="macos.debug" ;;
  *) fallar "sistema operativo no contemplado: $(uname -s)." ;;
esac

# Ruta declarada, relativa a la raíz: `res://` es `godot/`, así que `res://../x` es `x`.
relativa="$(tr -d '\r' < "$extension" \
  | sed -n "s|^$clave[[:space:]]*=[[:space:]]*\"res://\.\./\(target/debug/[^\"]*\)\"[[:space:]]*\$|\1|p")"
[ -n "$relativa" ] \
  || fallar "godot/heliarca.gdextension no declara '$clave' como \"res://../target/debug/...\"."
biblioteca="$raiz/$relativa"

rm -f "$biblioteca"
# --target-dir explícito: el .gdextension apunta al `target/` del workspace.
(cd "$raiz" && cargo build --locked -p heliarca-godot --target-dir target) \
  || fallar "el cdylib 'heliarca-godot' no compila."

[ -s "$biblioteca" ] \
  || fallar "la compilación terminó bien pero no generó '$relativa' (¿cambió el crate-type o el nombre de la biblioteca?)."

echo "OK: 'heliarca-godot' compila y generó $relativa."
