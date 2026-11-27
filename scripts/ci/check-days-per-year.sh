#!/usr/bin/env bash
# Evita la regresión de #439: el largo del año de una partida es `Params::days_per_year`, así que
# la constante `DAYS_PER_YEAR` (sólo su valor por defecto) no puede usarse en código Rust de
# producción bajo `crates/`. Permitidos:
#   (a) crates/heliarca-sim/src/time.rs (la define) y params.rs (arma el valor por defecto);
#   (b) la reexportación `pub use time::{...};` de crates/heliarca-sim/src/lib.rs;
#   (c) código de pruebas: `crates/*/tests/`, `crates/*/benches/` y los módulos `#[cfg(test)]`
#       (en línea, o en archivo aparte declarado con `#[cfg(test)] mod x;`).
# Se busca el identificador como palabra completa (no `MAX_DAYS_PER_YEAR` ni `days_per_year`),
# ignorando comentarios (incluidos doc-comments y doctests), strings y literales de carácter.
# No se busca el literal 365.
# Limitaciones (awk POSIX, sin parser de Rust): no expande macros ni ve capturas en formatos
# (`"{DAYS_PER_YEAR}"`); sólo `cfg(test)` y `cfg(all(..test..))` sin paréntesis anidados cuentan
# como pruebas (`any`/`not` son producción), y un `#[cfg(test)]` sobre un `fn` o `use` no exime.
# Uso: scripts/ci/check-days-per-year.sh [raiz-del-repo]   (por defecto, el directorio actual)
set -euo pipefail

raiz="${1:-.}"
sim="crates/heliarca-sim/src"

fallar() { echo "ERROR: $*" >&2; exit 1; }

for f in time.rs params.rs lib.rs; do
  [ -f "$raiz/$sim/$f" ] || fallar "no existe $sim/$f."
done

# Imprime el fuente Rust sin comentarios, strings ni literales de carácter, conservando los
# saltos de línea (los números de línea siguen siendo válidos). Igual que en check-sim-no-unsafe.sh.
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

# Sobre el fuente ya limpio imprime "USO <línea>" por cada uso fuera de módulos de prueba y
# "MOD <nombre>" por cada `#[cfg(test)] mod nombre;`. Con reexp=1 se omite `pub use time::...;`.
analizar() {
  awk -v reexp="$1" '
  function es_prueba(a) {
    return a == "[cfg(test)]" || a ~ /^\[cfg\(all\(([^()]*,)?test(,[^()]*)?\)\)\]$/
  }
  {
    s = $0
    while (s != "") {
      if (match(s, /^[ \t\r]+/)) { s = substr(s, RLENGTH + 1); continue }
      if (!match(s, /^[A-Za-z0-9_]+/)) RLENGTH = 1
      t = substr(s, 1, RLENGTH); s = substr(s, RLENGTH + 1)

      if (salto > 0) {                       # dentro de un módulo de prueba
        if (t == "{") salto++
        else if (t == "}") salto--
        continue
      }
      if (t == "DAYS_PER_YEAR" && !en_reexp) print "USO " NR

      if (numeral) {                         # tras "#": "[" o "![" abren un atributo
        if (t == "!") { interno = 1; continue }
        numeral = 0
        if (t == "[") { prof = 1; atr = "["; continue }
      }
      if (prof > 0) {
        atr = atr t
        if (t == "[") prof++
        else if (t == "]" && --prof == 0) {
          if (!interno && es_prueba(atr)) prueba = 1
          if (!interno && atr ~ /^\[path=/) ruta = 1
        }
        continue
      }
      if (t == "#") { numeral = 1; interno = 0; continue }

      if (reexp && ant2 == "pub" && ant == "use" && t == "time") en_reexp = 1
      if (en_reexp && t == ";") en_reexp = 0
      ant2 = ant; ant = t

      if (nombre != "") {                    # tras "#[cfg(test)] mod nombre"
        if (t == "{") salto = 1
        else if (t == ";" && !ruta) print "MOD " nombre
        nombre = ""; prueba = 0; ruta = 0
      } else if (es_mod) { nombre = t; es_mod = 0 }
      else if (prueba && t == "mod") es_mod = 1
      else if (t !~ /^(pub|crate|super|self|in|\(|\)|:)$/) { prueba = 0; ruta = 0 }
    }
  }'
}

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
: > "$tmp/usos"
: > "$tmp/pruebas"

find "$raiz/crates" -name target -prune -o -type f -name '*.rs' -print | sort > "$tmp/archivos"

while IFS= read -r archivo; do
  rel="${archivo#"$raiz"/}"
  case "$rel" in
    crates/*/src/*) ;;
    crates/*/tests/* | crates/*/benches/*) continue ;;
  esac
  reexp=0
  [ "$rel" = "$sim/lib.rs" ] && reexp=1
  limpiar "$archivo" | analizar "$reexp" > "$tmp/salida"

  # Directorio donde viven los submódulos declarados por este archivo.
  case "$rel" in
    */lib.rs | */main.rs | */mod.rs) base="${rel%/*}" ;;
    *) base="${rel%.rs}" ;;
  esac
  while read -r tipo dato; do
    case "$tipo" in
      MOD) printf '%s\n' "$base/$dato" >> "$tmp/pruebas" ;;
      USO)
        case "$rel" in
          "$sim/time.rs" | "$sim/params.rs") ;;
          *) printf '%s %s\n' "$dato" "$rel" >> "$tmp/usos" ;;
        esac
        ;;
    esac
  done < "$tmp/salida"
done < "$tmp/archivos"

errores=0
while read -r linea rel; do
  exento=0
  while IFS= read -r modulo; do
    case "$rel" in
      "$modulo.rs" | "$modulo"/*) exento=1 ;;
    esac
  done < "$tmp/pruebas"
  [ "$exento" -eq 0 ] || continue
  echo "ERROR: $rel:$linea: usa DAYS_PER_YEAR: $(sed -n "${linea}p" "$raiz/$rel" | sed -E 's/^[[:space:]]+//')" >&2
  errores=$((errores + 1))
done < "$tmp/usos"

if [ "$errores" -gt 0 ]; then
  fallar "$errores uso(s) de DAYS_PER_YEAR en código de producción (#439): usá Params::days_per_year o Calendar::days_per_year."
fi

echo "OK: DAYS_PER_YEAR no se usa en código de producción fuera de time.rs, params.rs y la reexportación de lib.rs."
