#!/usr/bin/env bash
# Verifica que `heliarca-sim` no contenga código `unsafe` (complementa a #7).
# `#![forbid(unsafe_code)]` en lib.rs solo cubre el lib y sus tests unitarios; los binarios
# (src/main.rs, src/bin), tests de integración, benches, examples y build.rs rigen solo con el
# `deny` del workspace, que un `#![allow(unsafe_code)]` relaja. Este chequeo cierra ese hueco:
#   (a) crates/heliarca-sim/src/lib.rs contiene `#![forbid(unsafe_code)]` (como código, no comentado)
#   (b) ningún .rs bajo crates/heliarca-sim/ usa la palabra `unsafe` como token de código
#       (bloque, fn, impl, trait, extern) ni menciona `unsafe_code` salvo dentro de forbid(...)/deny(...)
#   (c) el Cargo.toml de heliarca-sim sigue con `[lints] workspace = true`, sin `[lints.*]` propio
#       ni referencias a `unsafe_code`
# Estrategia: se eliminan comentarios (// y /* */ anidados, incluidos doc-comments), strings
# ("...", r#"..."#) y literales de carácter, y sobre el resto se busca `\bunsafe\b`.
# Limitaciones: no expande macros (un `macro_rules!` que genere `unsafe` solo se detecta si el
# token aparece escrito en el fuente) y no sigue `#[path]` ni `include!` hacia fuera del crate.
# Uso: scripts/ci/check-sim-no-unsafe.sh [raiz-del-repo]   (por defecto, el directorio actual)
set -euo pipefail

raiz="${1:-.}"
crate="$raiz/crates/heliarca-sim"
lib="$crate/src/lib.rs"
manifiesto="$crate/Cargo.toml"

fallar() { echo "ERROR: $*" >&2; exit 1; }

[ -d "$crate" ] || fallar "no existe $crate."
[ -f "$lib" ] || fallar "no existe $lib."
[ -f "$manifiesto" ] || fallar "no existe $manifiesto."

# Imprime el fuente Rust sin comentarios, strings ni literales de carácter, conservando los
# saltos de línea (los números de línea siguen siendo válidos).
limpiar() {
  awk '
  BEGIN { blk = 0; cad = 0; raw = -1 }
  function ident(c) { return c ~ /[A-Za-z0-9_]/ }
  {
    s = $0; n = length(s); out = ""; i = 1
    while (i <= n) {
      c = substr(s, i, 1); d = substr(s, i + 1, 1)
      if (blk > 0) {
        if (c == "/" && d == "*") { blk++; i += 2 }
        else if (c == "*" && d == "/") { blk--; i += 2 }
        else i++
        continue
      }
      if (raw >= 0) {
        h = substr(s, i + 1, raw); ok = (c == "\"" && length(h) == raw)
        if (ok) { g = h; gsub(/#/, "", g); ok = (g == "") }
        if (ok) { raw = -1; i += 1 + raw_len; } else i++
        continue
      }
      if (cad) {
        if (c == "\\") i += 2
        else if (c == "\"") { cad = 0; i++ }
        else i++
        continue
      }
      if (c == "/" && d == "/") break
      if (c == "/" && d == "*") { blk = 1; i += 2; continue }
      if (c == "\"") { cad = 1; i++; continue }
      p = (i > 1) ? substr(s, i - 1, 1) : ""
      if (c == "r" && (!ident(p) || p == "b")) {
        j = i + 1; h = 0
        while (substr(s, j, 1) == "#") { h++; j++ }
        if (substr(s, j, 1) == "\"") { raw = h; raw_len = h; i = j + 1; continue }
      }
      if (c == "\x27") {
        # Literal de carácter: \x27x\x27 o \x27\...\x27; si no, es un lifetime.
        if (d == "\\") {
          k = index(substr(s, i + 3), "\x27")
          if (k > 0) { i += 3 + k; continue }
        } else if (substr(s, i + 2, 1) == "\x27") { i += 3; continue }
      }
      out = out c; i++
    }
    print out
  }' "$1"
}

errores=0
marcar() { echo "ERROR: $*" >&2; errores=$((errores + 1)); }

# (a) forbid(unsafe_code) en lib.rs, como código.
if ! limpiar "$lib" | tr -d '[:space:]' | grep -qF '#![forbid(unsafe_code)]'; then
  marcar "$lib no contiene #![forbid(unsafe_code)] (activo, no comentado)."
fi

# (b) Ningún `unsafe` de código en ningún .rs del crate (lib, bin, tests, benches, examples, build.rs).
while IFS= read -r -d '' archivo; do
  hallazgos="$(limpiar "$archivo" \
    | sed -E 's/(forbid|deny)[[:space:]]*\([[:space:]]*unsafe_code[[:space:]]*\)//g' \
    | grep -nE '(^|[^A-Za-z0-9_])unsafe([^A-Za-z0-9_]|$)|(^|[^A-Za-z0-9_])unsafe_code([^A-Za-z0-9_]|$)' || true)"
  if [ -n "$hallazgos" ]; then
    while IFS= read -r linea; do
      marcar "$archivo:${linea%%:*}: 'unsafe' o 'unsafe_code' fuera de forbid/deny: $(sed -n "${linea%%:*}p" "$archivo" | sed -E 's/^[[:space:]]+//')"
    done <<<"$hallazgos"
  fi
done < <(find "$crate" -name target -prune -o -type f -name '*.rs' -print0)

# (c) Cargo.toml de heliarca-sim: hereda los lints del workspace y no relaja unsafe_code.
if grep -qE '^[[:space:]]*\[+[[:space:]]*lints[[:space:]]*\.' "$manifiesto"; then
  marcar "$manifiesto define una tabla [lints.*] propia; debe usar solo '[lints] workspace = true'."
fi
if grep -vE '^[[:space:]]*#' "$manifiesto" | grep -q 'unsafe_code'; then
  marcar "$manifiesto menciona unsafe_code; no se puede relajar el lint en el manifiesto."
fi
if ! awk '
  /^[[:space:]]*\[/ { en = ($0 ~ /^[[:space:]]*\[lints\][[:space:]]*(#.*)?$/); next }
  en && /^[[:space:]]*workspace[[:space:]]*=[[:space:]]*true[[:space:]]*(#.*)?$/ { ok = 1 }
  END { exit !ok }' "$manifiesto"; then
  marcar "$manifiesto no tiene '[lints] workspace = true'."
fi

if [ "$errores" -gt 0 ]; then
  fallar "heliarca-sim debe ser 100% safe ($errores problema(s)); ver #7: forbid(unsafe_code) en lib.rs y sin unsafe en bin/tests/benches/examples."
fi

echo "OK: heliarca-sim no contiene unsafe (forbid en lib.rs, sin unsafe en ningún .rs, lints heredados del workspace)."
