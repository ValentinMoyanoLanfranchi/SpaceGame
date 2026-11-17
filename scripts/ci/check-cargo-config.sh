#!/usr/bin/env bash
# Protege `.cargo/config.toml` y los alias de cargo que el README documenta (complementa a #8).
#   (a) `.cargo/config.toml` existe y su tabla [alias] define `c`, `t` y `l`.
#   (b) No hay un linker ni flags de enlace ACTIVOS en [build] ni en [target.*]:
#       claves `linker`, o `rustflags` que contengan `link-arg`/`link-args`/`linker=`. Un linker
#       fijado acá sería obligatorio también en el CI y en las máquinas de otros, y rompería los
#       runners que no lo tengan. Regla estricta: para probar un linker se usa una variable de
#       entorno (ver el comentario de config.toml). Excepción explícita: una línea con el
#       comentario `# ci-allow-linker` queda exenta (decisión consciente y visible en el diff).
#   (c) Todo `cargo X` de una sola letra que el README cite entre comillas invertidas
#       es un alias definido en [alias] (el README no puede documentar un alias inexistente).
# Estrategia: se parsea el TOML línea a línea con awk, sin comentarios (`#` fuera de strings),
# siguiendo la tabla actual ([alias], [target.*], etc.).
# Limitaciones: no es un parser TOML completo; no entiende la forma con puntos en la raíz
# (`alias.c = "..."`), tablas inline ni strings multilínea. Para (b), `rustflags` multilínea se
# revisa hasta el `]` que la cierra. Solo se mira `.cargo/config.toml` de la raíz (no
# `config` sin extensión ni ~/.cargo/config.toml). Para (c), se ignoran los subcomandos de
# varias letras (cargo test, cargo clippy...) y los alias integrados de cargo b y r.
# Uso: scripts/ci/check-cargo-config.sh [raiz-del-repo]   (por defecto, el directorio actual)
set -euo pipefail

raiz="${1:-.}"
config="$raiz/.cargo/config.toml"
readme="$raiz/README.md"

fallar() { echo "ERROR: $*" >&2; exit 1; }

[ -f "$config" ] || fallar "no existe $config (el README documenta los alias de cargo c, t y l)."

# Quita el comentario de una línea (un '#' fuera de comillas) y el \r de CRLF.
# Se reutiliza en los dos awk de abajo.
quitar_comentario='
function sin_comentario(s,   i, n, c, q, out) {
  sub(/\r$/, "", s); n = length(s); q = ""; out = ""
  for (i = 1; i <= n; i++) {
    c = substr(s, i, 1)
    if (q != "") { out = out c; if (c == "\\" && q == "\"") { out = out substr(s, i + 1, 1); i++ } else if (c == q) q = "" }
    else if (c == "\"" || c == "\047") { q = c; out = out c }
    else if (c == "#") break
    else out = out c
  }
  return out
}'

# Nombres de alias definidos en la tabla [alias] (uno por línea).
alias_definidos="$(awk "$quitar_comentario"'
  { l = sin_comentario($0) }
  l ~ /^[ \t]*\[/ { t = l; gsub(/[ \t]/, "", t); en_alias = (t ~ /^\[alias\]/); next }
  en_alias && match(l, /^[ \t]*("[^"]+"|[A-Za-z0-9_-]+)[ \t]*=/) {
    k = substr(l, 1, RLENGTH - 1); gsub(/[ \t"]/, "", k); print k
  }' "$config")"

# (a) alias obligatorios.
for a in c t l; do
  grep -qxF "$a" <<<"$alias_definidos" \
    || fallar "$config no define el alias '$a' en la tabla [alias] (el README documenta cargo $a)."
done

# (b) linker / flags de enlace activos en [build] y [target.*].
prohibido="$(awk "$quitar_comentario"'
  { crudo = $0; l = sin_comentario($0) }
  l ~ /^[ \t]*\[/ {
    t = l; gsub(/[ \t]/, "", t); en_zona = (t ~ /^\[(build|target\.)/); en_flags = 0; next
  }
  !en_zona { next }
  crudo ~ /# *ci-allow-linker/ { if (l ~ /rustflags[ \t]*=[ \t]*\[/ && l !~ /\]/) en_flags = 1; else if (l ~ /\]/) en_flags = 0; next }
  l ~ /^[ \t]*linker[ \t]*=/ { printf "%d: %s\n", NR, l; next }
  l ~ /^[ \t]*rustflags[ \t]*=/ { en_flags = (l !~ /\]/ && l ~ /\[/); if (l ~ /link-args?|linker[ \t]*=/) printf "%d: %s\n", NR, l; next }
  en_flags { if (l ~ /link-args?|linker[ \t]*=/) printf "%d: %s\n", NR, l; if (l ~ /\]/) en_flags = 0 }
' "$config")"

if [ -n "$prohibido" ]; then
  echo "$prohibido" >&2
  fallar "$config activa un linker o link-arg en [build]/[target.*]; rompería los runners que no lo tengan. Probalo con variables de entorno, o dejalo comentado, o marcá la línea con '# ci-allow-linker' si es deliberado."
fi

# (c) alias de una letra citados en el README (`cargo X`) deben existir.
if [ -f "$readme" ]; then
  citados="$(grep -oE '`cargo [a-z]`' "$readme" | sed -E 's/`cargo ([a-z])`/\1/' | sort -u || true)"
  for a in $citados; do
    case "$a" in b|r) continue ;; esac   # alias integrados de cargo (build, run)
    grep -qxF "$a" <<<"$alias_definidos" \
      || fallar "el README menciona 'cargo $a' pero $config no define ese alias."
  done
fi

echo "OK: .cargo/config.toml define los alias c, t y l, sin linker activo, y el README no cita alias inexistentes."
