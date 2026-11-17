#!/usr/bin/env bash
# Evita la deriva entre los clippy.toml del repo (#19/#20). Clippy usa el PRIMER clippy.toml
# hacia arriba desde cada crate y no fusiona archivos, así que hay dos configs que deben
# mantenerse alineadas: `clippy.toml` (raíz; rige heliarca-cli y heliarca-godot) y
# `crates/heliarca-sim/clippy.toml` (msrv + std::time + colecciones no deterministas).
# Falla si:
#  (a) el `msrv` de cualquier clippy.toml (raíz y `crates/*/clippy.toml`) no coincide con
#      `rust-version` de `[workspace.package]` en el Cargo.toml raíz (o falta);
#  (b) alguna entrada `path = "std::time::..."` del clippy.toml raíz (en disallowed-types o
#      disallowed-methods) no está en el de heliarca-sim: el de sim debe ser superconjunto;
#  (c) el de heliarca-sim perdió la prohibición de std::collections::HashMap o HashSet
#      (en disallowed-types), que es el criterio de #20;
#  (d) algún crate bajo `crates/` (directorio con Cargo.toml) no tiene clippy.toml propio ni
#      raíz que heredar (basta cualquiera de los dos).
# Limitaciones del parseo (awk POSIX, sin parser TOML): se leen sólo líneas con `msrv = "x"`
# y entradas `path = "..."` (comillas dobles o simples) en una misma línea, asignadas a la
# clave `disallowed-*` vigente; se ignoran líneas comentadas y todo lo que sigue a un `#`
# previo en la línea; no se entienden cadenas multilínea, `path` fuera de una tabla inline ni
# `.clippy.toml` (con punto). Los `reason` no se comparan, sólo los paths. Se toleran CRLF,
# espacios y distinto orden de entradas.
# Uso: scripts/ci/check-clippy-config.sh [raiz-del-repo]   (por defecto, el directorio actual)
set -euo pipefail

raiz="${1:-.}"
sim_rel="crates/heliarca-sim/clippy.toml"

errores=0
fallar() { echo "ERROR: $*" >&2; errores=$((errores + 1)); }

[ -f "$raiz/Cargo.toml" ] || { echo "ERROR: no existe Cargo.toml en '$raiz'." >&2; exit 1; }

# Extrae de un clippy.toml líneas "msrv<TAB>valor" y "path<TAB>clave<TAB>path".
extraer_clippy() {
  awk '
    { sub(/\r$/, "") }
    /^[ \t]*#/ { next }
    /^[ \t]*disallowed-[a-z-]+[ \t]*=/ {
      clave = $0; sub(/^[ \t]*/, "", clave); sub(/[ \t]*=.*/, "", clave); actual = clave
    }
    /^[ \t]*[A-Za-z_-]+[ \t]*=/ && $0 !~ /^[ \t]*disallowed-/ && $0 !~ /^[ \t]*\{/ { actual = "" }
    /^[ \t]*msrv[ \t]*=/ {
      if (match($0, /"[^"]*"|\047[^\047]*\047/)) {
        v = substr($0, RSTART + 1, RLENGTH - 2)
        antes = substr($0, 1, RSTART - 1)
        if (antes !~ /#/) print "msrv\t" v
      }
      next
    }
    {
      resto = $0; base = 0
      while (match(resto, /(^|[^A-Za-z0-9_-])path[ \t]*=[ \t]*("[^"]*"|\047[^\047]*\047)/)) {
        trozo = substr(resto, RSTART, RLENGTH)
        antes = substr($0, 1, base + RSTART - 1)
        if (antes ~ /#/) break
        sub(/^[^p]*path[ \t]*=[ \t]*/, "", trozo)
        print "path\t" actual "\t" substr(trozo, 2, length(trozo) - 2)
        base += RSTART + RLENGTH - 1
        resto = substr(resto, RSTART + RLENGTH)
      }
    }
  ' "$1"
}

# rust-version de [workspace.package] en el Cargo.toml raíz.
rust_version="$(awk '
  { sub(/\r$/, "") }
  /^[ \t]*#/ { next }
  /^[ \t]*\[/ { s = $0; gsub(/[ \t]/, "", s); en = (s ~ /^\[workspace\.package\]/); next }
  en && /^[ \t]*rust-version[ \t]*=/ {
    if (match($0, /"[^"]*"/)) { print substr($0, RSTART + 1, RLENGTH - 2); exit }
  }
' "$raiz/Cargo.toml")"
[ -n "$rust_version" ] || { echo "ERROR: no se encontró rust-version en [workspace.package] de $raiz/Cargo.toml." >&2; exit 1; }

# Todos los clippy.toml: raíz y por crate.
archivos=()
[ -f "$raiz/clippy.toml" ] && archivos+=("clippy.toml")
for f in "$raiz"/crates/*/clippy.toml; do
  [ -f "$f" ] && archivos+=("${f#"$raiz"/}")
done

# (a) msrv de cada clippy.toml == rust-version.
for rel in "${archivos[@]}"; do
  msrv="$(extraer_clippy "$raiz/$rel" | awk -F'\t' '$1 == "msrv" { print $2; exit }')"
  if [ -z "$msrv" ]; then
    fallar "$rel no define 'msrv'; debe ser $rust_version (rust-version de [workspace.package])."
  elif [ "$msrv" != "$rust_version" ]; then
    fallar "$rel: msrv = \"$msrv\" difiere de rust-version = \"$rust_version\" de [workspace.package] en Cargo.toml."
  fi
done

# (b) las entradas std::time del raíz deben estar en el de sim.
if [ ! -f "$raiz/clippy.toml" ]; then
  fallar "falta clippy.toml en la raíz (rige heliarca-cli y heliarca-godot)."
elif [ ! -f "$raiz/$sim_rel" ]; then
  fallar "falta $sim_rel (heliarca-sim necesita su propio clippy.toml con las reglas del raíz y las colecciones)."
else
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  extraer_clippy "$raiz/clippy.toml" | awk -F'\t' '$1 == "path" && $3 ~ /^std::time::/ { print $2 "\t" $3 }' | sort -u > "$tmp/raiz"
  extraer_clippy "$raiz/$sim_rel" | awk -F'\t' '$1 == "path" { print $2 "\t" $3 }' | sort -u > "$tmp/sim"
  faltan="$(comm -23 "$tmp/raiz" "$tmp/sim" || true)"
  if [ -n "$faltan" ]; then
    while IFS="$(printf '\t')" read -r clave ruta; do
      [ -n "$ruta" ] || continue
      fallar "$sim_rel no tiene { path = \"$ruta\" } en ${clave:-?} y clippy.toml (raíz) sí: el de sim debe ser superconjunto del raíz."
    done <<<"$faltan"
  fi

  # (c) HashMap/HashSet siguen prohibidos en sim.
  sim_paths="$(extraer_clippy "$raiz/$sim_rel" | awk -F'\t' '$1 == "path" && $2 == "disallowed-types" { print $3 }')"
  for tipo in std::collections::HashMap std::collections::HashSet; do
    if ! grep -qxF "$tipo" <<<"$sim_paths"; then
      fallar "$sim_rel perdió { path = \"$tipo\" } en disallowed-types: es el criterio de #20 (colecciones no deterministas en heliarca-sim)."
    fi
  done
fi

# (d) todo crate bajo crates/ tiene clippy.toml propio o hereda el raíz.
for d in "$raiz"/crates/*/; do
  [ -f "${d}Cargo.toml" ] || continue
  if [ ! -f "${d}clippy.toml" ] && [ ! -f "$raiz/clippy.toml" ]; then
    fallar "${d#"$raiz"/} no tiene clippy.toml propio ni hay clippy.toml en la raíz para heredar."
  fi
done

if [ "$errores" -gt 0 ]; then
  echo "FALLÓ: $errores problema(s) de deriva en los clippy.toml (ver README/CONTRIBUTING: clippy no fusiona archivos)." >&2
  exit 1
fi

echo "OK: los clippy.toml están alineados (msrv $rust_version, std::time y HashMap/HashSet en sim)."
