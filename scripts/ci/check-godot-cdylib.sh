#!/usr/bin/env bash
# Protege la verificación del `cdylib` de Godot (#61): si alguien la saca o la debilita en
# `ci.yml`, esto falla. Exige que el job `build-test` (que `validate` agrega, ver
# check-workflow-aggregator.sh):
#   (a) corra `bash scripts/build-godot-cdylib.sh` (línea exacta: sin `|| true` ni agregados)
#       en un paso sin `if:`, `continue-on-error:` ni `shell:` propios;
#   (b) no tenga `if:` ni `continue-on-error:` de job;
#   (c) use una matriz con `ubuntu-latest` y `windows-latest`, sin `exclude:`.
# Parseo: awk POSIX, YAML en estilo bloque con la indentación de dos espacios de `ci.yml`.
# Uso: scripts/ci/check-godot-cdylib.sh [raiz-del-repo]   (por defecto, el directorio actual)
set -euo pipefail

raiz="${1:-.}"
ci="$raiz/.github/workflows/ci.yml"

fallar() { echo "ERROR: $*" >&2; exit 1; }

[ -f "$ci" ] || fallar "no existe .github/workflows/ci.yml."
[ -f "$raiz/scripts/build-godot-cdylib.sh" ] || fallar "no existe scripts/build-godot-cdylib.sh."

problemas="$(awk '
  function cerrar_paso() {
    if (es_paso) { encontrado = 1; if (debil != "") print "el paso que verifica el cdylib no puede tener:" debil }
    es_paso = 0; debil = ""
  }
  { sub(/\r$/, "") }
  /^[ \t]*(#.*)?$/ { next }
  /^[^ ]/ { cerrar_paso(); enjob = 0; next }
  /^  [^ ]/ { cerrar_paso(); enjob = ($0 ~ /^  build-test:[ \t]*(#.*)?$/); next }
  !enjob { next }
  {
    sub(/[ \t]+#.*$/, ""); sub(/[ \t]+$/, "")
    if ($0 ~ /^      - /) { cerrar_paso(); sub(/^      - /, "        ") }
    if ($0 ~ /^    [^ ]/) {
      cerrar_paso()
      if ($0 ~ /^    (if|continue-on-error):/) print "el job build-test no puede tener:" $0
    }
    if ($0 ~ /^        run: bash scripts\/build-godot-cdylib\.sh$/) es_paso = 1
    if ($0 ~ /^        (if|continue-on-error|shell):/) { k = $0; sub(/^ +/, " ", k); debil = debil k }
    if ($0 ~ /^ +os:.*ubuntu-latest/ && $0 ~ /windows-latest/) matriz = 1
    if ($0 ~ /^ +exclude:/) print "la matriz de build-test no puede tener exclude:"
  }
  END {
    cerrar_paso()
    if (!encontrado) print "build-test no tiene un paso con \"run: bash scripts/build-godot-cdylib.sh\"."
    if (!matriz) print "la matriz de build-test debe incluir ubuntu-latest y windows-latest."
  }' "$ci")"

if [ -n "$problemas" ]; then
  echo "$problemas" >&2
  fallar "ci.yml ya no garantiza que 'validate' falle si el cdylib de Godot no compila."
fi

echo "OK: build-test compila y verifica el cdylib de Godot en Linux y Windows."
