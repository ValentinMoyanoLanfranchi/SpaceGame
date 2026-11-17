#!/usr/bin/env bash
# Protege el agregador `validate` (el check requerido por el ruleset "Proteger main") y la
# seguridad básica de los workflows (complementa a #10).
#   (a) `ci.yml` tiene un job con id Y name `validate` (el contexto que exige el ruleset).
#   (b) Todo otro job de `ci.yml` figura en el `needs:` de `validate`; si no, un job rojo
#       no bloquearía el merge.
#   (c) `validate` corre con `if: always()` (si no, se saltea cuando falla un job y GitHub
#       lo da por "skipped"=aprobado), no tiene `continue-on-error` y consulta
#       `needs.<job>.result` de cada job del que depende y compara contra `success`.
#   (d) En cada workflow: `permissions` de nivel workflow es `contents: read` (sin escritura),
#       y no hay `write-all`, `pull_request_target` ni `secrets: inherit`.
#   (e) En cada workflow: toda línea `uses:` está fijada a un SHA de 40 hex (o es una acción
#       local `./...`, o una imagen `docker://...@sha256:<64 hex>`).
# Excepción explícita (d)/(e): una línea con el comentario `# ci-allow-insecure` queda exenta
# (decisión consciente y visible en el diff).
#
# Parseo: awk puro (POSIX; sin yq ni python, que no están garantizados en todos los entornos).
# Limitaciones: asume YAML en estilo bloque con indentación de espacios (el de GitHub Actions
# típico). Soporta `needs:` escalar, en lista de flujo en UNA línea (`[a, b]`) o en lista de
# bloque (`- a`); una lista de flujo partida en varias líneas, anclas/alias y claves entre
# comillas se rechazan o no se interpretan (el script falla con un mensaje en vez de pasar).
# Los comentarios se quitan con la regla simple " #..." (un `#` precedido de espacio dentro
# de un string entre comillas se perdería; no afecta a los patrones que se validan).
# Uso: scripts/ci/check-workflow-aggregator.sh [raiz-del-repo]   (por defecto, el directorio actual)
set -euo pipefail

raiz="${1:-.}"
dir="$raiz/.github/workflows"
ci="$dir/ci.yml"

fallar() { echo "ERROR: $*" >&2; exit 1; }

[ -d "$dir" ] || fallar "no existe .github/workflows en la raíz del repo."
[ -f "$ci" ] || fallar "no existe .github/workflows/ci.yml."

errores=0
error() { echo "ERROR: $*" >&2; errores=$((errores + 1)); }

# Emite un registro por línea relevante: JOB/NAME/NEEDS/IF/COE/REF/PERMVAL/PERM/USES/RISK/BAD.
analizar() {
  awk '
  function quitar(v) {
    sub(/\r$/, "", v)
    sub(/[ \t]+#.*$/, "", v)
    sub(/^[ \t]+/, "", v); sub(/[ \t]+$/, "", v)
    if (v ~ /^".*"$/ || v ~ /^\047.*\047$/) v = substr(v, 2, length(v) - 2)
    return v
  }
  { sub(/\r$/, "") }
  /^[ \t]*$/ { next }
  /^[ \t]*#/ { next }
  {
    linea = $0
    exento = (linea ~ /# *ci-allow-insecure/)
    match(linea, /[^ ]/); ind = RSTART - 1
    if (linea ~ /^\t/ || linea ~ /^ *\t/) { print "BAD\tindentación con tabuladores (no soportada)"; next }

    # Riesgos en cualquier parte del archivo.
    if (!exento) {
      if (linea ~ /(^|[^A-Za-z0-9_-])pull_request_target([^A-Za-z0-9_-]|$)/) print "RISK\tpull_request_target: " linea
      if (linea ~ /write-all/) print "RISK\tpermissions write-all: " linea
      if (linea ~ /^[ \t-]*secrets:[ \t]*inherit/) print "RISK\tsecrets: inherit: " linea
    }

    # Acciones usadas.
    if (linea ~ /^[ \t-]*uses:[ \t]*/) {
      v = linea; sub(/^[ \t-]*uses:[ \t]*/, "", v); v = quitar(v)
      print "USES\t" (exento ? "ok" : "chk") "\t" v
    }

    if (ind == 0) {
      enjobs = (linea ~ /^jobs:[ \t]*(#.*)?$/)
      enperm = 0; jobactual = ""; jobindent = 0; ennedds = 0
      if (linea ~ /^permissions:/) {
        v = linea; sub(/^permissions:[ \t]*/, "", v); v = quitar(v)
        if (v != "") print "PERMVAL\t" v; else { enperm = 1; print "PERMBLOCK" }
      }
      next
    }
    if (enperm) {
      k = linea; sub(/^[ \t]+/, "", k)
      n = index(k, ":"); if (n > 0) print "PERM\t" substr(k, 1, n - 1) "\t" quitar(substr(k, n + 1))
      next
    }
    if (!enjobs) next

    if (jobindent == 0) jobindent = ind
    if (ind == jobindent) {
      ennedds = 0
      k = linea; sub(/^[ \t]+/, "", k)
      if (k ~ /^[A-Za-z_][A-Za-z0-9_-]*:[ \t]*(#.*)?$/) { jobactual = k; sub(/:.*$/, "", jobactual); print "JOB\t" jobactual }
      else { print "BAD\tclave de job no interpretable: " k }
      next
    }
    if (jobactual == "") next

    # Referencias a needs.<job>.result en cualquier línea del job.
    resto = linea
    while (match(resto, /needs\.[A-Za-z0-9_-]+\.result/)) {
      r = substr(resto, RSTART + 6, RLENGTH - 13)
      print "REF\t" jobactual "\t" r
      resto = substr(resto, RSTART + RLENGTH)
    }
    if (linea ~ /success/) print "SUCCESS\t" jobactual

    if (ind == jobindent + 2 || ind > jobindent) {
      if (ennedds && ind > jobindent + 2 && linea ~ /^[ \t]*-[ \t]/) {
        v = linea; sub(/^[ \t]*-[ \t]*/, "", v); print "NEEDS\t" jobactual "\t" quitar(v); next
      }
      if (ind == jobindent + 2) {
        ennedds = 0
        k = linea; sub(/^[ \t]+/, "", k)
        n = index(k, ":"); if (n == 0) next
        clave = substr(k, 1, n - 1); v = quitar(substr(k, n + 1))
        if (clave == "name") print "NAME\t" jobactual "\t" v
        else if (clave == "if") print "IF\t" jobactual "\t" v
        else if (clave == "continue-on-error") print "COE\t" jobactual "\t" v
        else if (clave == "needs") {
          if (v == "") ennedds = 1
          else if (v ~ /^\[.*\]$/) {
            v = substr(v, 2, length(v) - 2); m = split(v, partes, /[ \t]*,[ \t]*/)
            for (i = 1; i <= m; i++) { p = quitar(partes[i]); if (p != "") print "NEEDS\t" jobactual "\t" p }
          } else if (v ~ /^\[/) print "BAD\tneeds de " jobactual " en lista de flujo multilínea (no soportada)"
          else print "NEEDS\t" jobactual "\t" v
        }
      }
    }
  }' "$1"
}

# --- (d) y (e): todos los workflows ---------------------------------------------------------
archivos=()
for f in "$dir"/*.yml "$dir"/*.yaml; do [ -f "$f" ] && archivos+=("$f"); done
[ "${#archivos[@]}" -gt 0 ] || fallar "no hay workflows (*.yml|*.yaml) en .github/workflows."

for f in "${archivos[@]}"; do
  nombre="${f#"$raiz"/}"
  datos="$(analizar "$f")"
  tiene_permiso=0
  while IFS=$'\t' read -r tipo a b c; do
    case "$tipo" in
      BAD) error "$nombre: $a" ;;
      RISK) error "$nombre: patrón inseguro no autorizado ($a). Si es deliberado, agregá '# ci-allow-insecure' en esa línea." ;;
      PERMVAL) tiene_permiso=1; error "$nombre: 'permissions: $a' de nivel workflow; debe ser un bloque con 'contents: read'." ;;
      PERMBLOCK) tiene_permiso=2 ;;
      PERM)
        if [ "$a" = "contents" ] && [ "$b" = "read" ]; then tiene_permiso=3
        elif [ "$b" != "none" ] && [ "$b" != "read" ]; then error "$nombre: permissions de nivel workflow con '$a: $b'; el token debe ser de solo lectura."
        fi ;;
      USES)
        if [ "$a" = "chk" ]; then
          ref="$b"
          if [[ "$ref" == ./* ]]; then :
          elif [[ "$ref" =~ ^docker://.+@sha256:[0-9a-f]{64}$ ]]; then :
          elif [[ "$ref" =~ ^[A-Za-z0-9._-]+/[A-Za-z0-9._/-]+@[0-9a-f]{40}$ ]]; then :
          else error "$nombre: 'uses: $ref' no está fijado a un SHA de 40 hex (ni es una acción local ./)."
          fi
        fi ;;
    esac
  done <<< "$datos"
  [ "$tiene_permiso" -eq 3 ] || error "$nombre: falta 'permissions:' de nivel workflow con 'contents: read'."
done

# --- (a), (b), (c): ci.yml ------------------------------------------------------------------
datos="$(analizar "$ci")"
jobs=(); declare -A nombres ifs coes needs refs exito
while IFS=$'\t' read -r tipo a b; do
  case "$tipo" in
    JOB) jobs+=("$a") ;;
    NAME) nombres["$a"]="$b" ;;
    IF) ifs["$a"]="$b" ;;
    COE) coes["$a"]="$b" ;;
    NEEDS) needs["$a"]="${needs[$a]:-} $b " ;;
    REF) refs["$a"]="${refs[$a]:-} $b " ;;
    SUCCESS) exito["$a"]=1 ;;
  esac
done <<< "$datos"

esta_job=0
for j in "${jobs[@]}"; do [ "$j" = "validate" ] && esta_job=1; done
if [ "$esta_job" -eq 0 ]; then
  error "ci.yml no tiene un job con id 'validate' (es el check requerido por el ruleset; no lo renombres ni lo borres)."
else
  [ "${nombres[validate]:-}" = "validate" ] \
    || error "el job 'validate' debe tener 'name: validate' (el contexto del check requerido); tiene '${nombres[validate]:-<sin name>}'."

  for j in "${jobs[@]}"; do
    [ "$j" = "validate" ] && continue
    [[ "${needs[validate]:-}" == *" $j "* ]] \
      || error "el job '$j' no está en 'needs:' de 'validate'; si falla, no bloquearía el merge."
    [[ "${refs[validate]:-}" == *" $j "* ]] \
      || error "'validate' no consulta 'needs.$j.result'; debe exigir 'success' de cada job."
  done

  [[ "${ifs[validate]:-}" == *"always()"* ]] \
    || error "el job 'validate' debe tener 'if: always()' (hoy: '${ifs[validate]:-<sin if>}'); sin eso se saltea cuando otro job falla."
  case "${coes[validate]:-false}" in
    false) ;;
    *) error "el job 'validate' no puede tener 'continue-on-error: ${coes[validate]}'." ;;
  esac
  [ -n "${exito[validate]:-}" ] || error "'validate' no compara el resultado contra 'success'."
fi

if [ "$errores" -gt 0 ]; then
  echo "FALLÓ: $errores problema(s) en los workflows." >&2
  exit 1
fi
echo "OK: 'validate' agrega todos los jobs de ci.yml (${jobs[*]}); permisos de solo lectura y acciones fijadas por SHA en ${#archivos[@]} workflow(s)."
