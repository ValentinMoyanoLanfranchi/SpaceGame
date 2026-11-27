# HELIARCA (nombre provisorio)

Un 4X espacial 2D por incrementos de tiempo, estilo **Aurora 4X** con economía mixta profunda, logística real y un mundo que sigue su curso. El jugador gobierna un Estado que parte de la **Tierra pre-espacial** y busca llegar al espacio y cruzar su primer punto de salto.

- **Núcleo de simulación:** Rust, determinista y dirigido por datos (JSON).
- **Interfaz:** Godot 4 + godot-rust (gdext).
- **Alcance actual:** el **MVP** = Fase 1 jugable (de la Tierra hasta cruzar el primer punto de salto).

## Documentación de diseño (GDD)
Está en [`Documentacion/`](Documentacion/). Puntos de entrada:

| Doc | Para qué |
|---|---|
| [00 Visión general](Documentacion/00_vision_general.md) | Qué es el juego y su índice |
| [17 Decisiones y roadmap](Documentacion/17_preguntas_abiertas_y_roadmap.md) | Decisiones tomadas, riesgos, preguntas abiertas |
| [21 Alcance del MVP y arquitectura](Documentacion/21_alcance_mvp_y_arquitectura.md) | Qué entra en el MVP y cómo se construye |
| [22 Datos iniciales del MVP](Documentacion/22_datos_iniciales_mvp.md) | Números concretos a cargar en JSON |
| [23 Plan de desarrollo](Documentacion/23_plan_de_desarrollo.md) | Cómo se ejecuta el MVP paso a paso (hitos, jerarquía, reglas) |
| [24 Decisión Godot y gdext](Documentacion/24_decision_godot_gdext.md) | Versiones fijadas de Godot 4 y gdext, licencias, riesgos |

## Requisitos
Para compilar el núcleo de simulación necesitás:

- **[rustup](https://rustup.rs/)**, el instalador de Rust. Al entrar al repo, rustup lee [`rust-toolchain.toml`](rust-toolchain.toml) y instala solo el toolchain fijado, con `rustfmt` y `clippy`.
- **Rust 1.100.0** como versión mínima (es también la fijada en `rust-toolchain.toml`).
- **Windows:** [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) con el workload **"Desarrollo para escritorio con C++"** (incluye MSVC y el Windows SDK). Es lo que rustup espera para el target `x86_64-pc-windows-msvc`.
- **Linux:** `build-essential` (por ejemplo, `sudo apt install build-essential` en Debian/Ubuntu).

Para comprobar que quedó bien, corré `cargo --version` desde la raíz del repo: tiene que informar `1.100.0`.

## Desarrollo local
Con los [requisitos](#requisitos) instalados, todo se corre desde la raíz del repo. Los comandos son los mismos en Windows (PowerShell) y en Linux.

```
git clone https://github.com/ValentinMoyanoLanfranchi/SpaceGame.git
cd SpaceGame
```

### Compilar, probar y verificar
| Qué | Comando |
|---|---|
| Compilar todo el workspace (como el CI) | `cargo build --workspace --locked` |
| Tests | `cargo test --workspace` |
| Lints (clippy, los warnings son error) | `cargo clippy --workspace --all-targets -- -D warnings` |
| Formato (solo verifica) | `cargo fmt --all -- --check` |
| Formato (aplica los cambios) | `cargo fmt --all` |

La primera corrida descarga el toolchain fijado en `rust-toolchain.toml`, así que tarda un poco más. Antes de abrir un PR, los cuatro primeros tienen que pasar en verde (es lo que valida el CI).

### Atajos de cargo
[`.cargo/config.toml`](.cargo/config.toml) define tres alias, ver [Tiempos de compilación](#tiempos-de-compilación):

- `cargo c`: verificación rápida (`check`) de todo el workspace, sin generar binarios.
- `cargo t`: los tests de todo el workspace.
- `cargo l`: el mismo clippy con `-D warnings` que corre el CI.

### Ejecutar la CLI
`cargo run -p heliarca-cli -- --help` lista los subcomandos y opciones del ejecutable sin interfaz (los argumentos van después de `--`):

- `run`: avanza la simulación N años e imprime fecha final, eventos, hash de estado y duración real (ver [Correr la simulación](#correr-la-simulación)); con `--script <archivo>` envía además los comandos de un script JSON (ver [Scripts de comandos](#scripts-de-comandos)) y con `--save <archivo>` guarda el estado final (ver [Guardar y cargar](#guardar-y-cargar)).
- `load <archivo>`: carga un guardado y continúa N años (ver [Guardar y cargar](#guardar-y-cargar)).
- `validate-data`: valida los datos de `--data-dir` (ver [Validar los datos](#validar-los-datos)).
- Opciones globales: `--seed <u64>` (por defecto `0`) y `--data-dir <ruta>` (por defecto `data`).
- Sin subcomando imprime un banner con la versión del núcleo y la fecha inicial (`heliarca-cli (sim 0.1.0) - Año 1, día 1, 00:00:00`).

Códigos de salida: `0` éxito (también `--help` y `--version`), `1` error de datos o de simulación, `2` error de uso (argumento inválido o desconocido).

### Correr la simulación
`cargo run -p heliarca-cli -- run --years 40 --seed 1` carga `--data-dir` (por defecto `data/`), avanza 40 años y resume la corrida:

```
Años simulados: 40
Semilla: 1
Incremento: d1
Pasos: 14600
Fecha final: Año 41, día 1, 00:00:00
Eventos: 0
Hash de estado: <32 hex>
Duración real: 0,002 s (7300000 pasos/s)
```

(El hash depende de la versión del formato del hash de estado, así que el ejemplo no lo fija; un test de la CLI comprueba que las líneas de este ejemplo coinciden con la salida real.)

Opciones de `run` (además de `--seed` y `--data-dir`):

- `--years <N>`: años a simular, de 0 a 10000 (por defecto `40`, el horizonte del criterio de rendimiento). Un año dura `days_per_year` días (`data/params.json`, 365 por defecto; en `load`, el del guardado), el mismo del calendario que muestra la fecha. `--years 0` es válido: 0 pasos. Fuera de rango es un error de uso (código 2).
- `--increment <s5|h1|d1|d5|d30>`: incremento de cada paso (por defecto `d1`). Si lo que falta es menos que el incremento, el último tramo se completa con incrementos menores (D30, D5, D1, H1, S5), así que nunca se pasa del total. Si el núcleo interrumpe un avance, la CLI sigue desde donde quedó; si un avance no progresa nada, corta con error (código 1) en vez de quedarse girando.

Los datos se cargan con la misma política completa que `validate-data` (archivos más registro de ids y referencias); si fallan, los errores salen por stderr, uno por línea, con código 1. Las líneas de la salida son estables y parseables (`Clave: valor`); todas salvo `Duración real` son deterministas: misma semilla, años e incremento dan el mismo hash. El hash incluye el estado interno (semilla/RNG...) pero **no** el contador de pasos: la misma partida avanzada con otros incrementos (p. ej. 30×`d1` o 1×`d30`) da el mismo hash, aunque `Pasos` difiera.

**Duración real y línea base de rendimiento.** Es el tiempo de pared del bucle de avance (sin la carga de datos), con tres decimales y coma decimal (`0,002 s`); si el tiempo es mayor que 0 se agregan los pasos por segundo (`(7300000 pasos/s)`), útil cuando la corrida es tan corta que el tiempo no tiene resolución. Se mide sólo en la CLI (`crates/heliarca-cli/src/timing.rs`) y no entra a la simulación ni al hash. Es la línea base para el criterio «40 años de juego en menos de 30 s» (doc 21 §1). Con el escenario actual (vacío, sin sistemas), 40 años a `d1` tardan en una máquina de desarrollo con Windows 11: 0,002 s en debug (`cargo run -p heliarca-cli -- run --years 40 --seed 1`) y menos de 1 ms en release (`cargo run --release -p heliarca-cli -- run --years 40 --seed 1`); a `s5` (252 millones de pasos en 40 años) tarda unos 3,9 s en release. **Ojo con los incrementos chicos y muchos años:** el costo es lineal en los pasos; `--years 10000 --increment s5` (≈6,3 mil millones de pasos) tarda unos 16 minutos. No hay tope por eso: elegí el incremento según lo que necesites medir. Son números de referencia que irán subiendo cuando existan los sistemas; el presupuesto se vigila contra el criterio de 30 s, no contra estos valores.

### Guardar y cargar
`run --save <archivo>` guarda el estado al terminar la corrida y `load <archivo> --years N` lo carga y sigue:

```
cargo run -p heliarca-cli -- run --years 10 --seed 1 --save partida.sav.zst
cargo run -p heliarca-cli -- load partida.sav.zst --years 30
```

- **Extensión y formato:** `.sav.zst` (el JSON del estado comprimido con zstd; la CLI no impone la extensión). La escritura es atómica: un fallo a mitad deja intacto el guardado anterior, y guardar sobre un archivo existente lo reemplaza. El guardado incluye la huella de `--data-dir`.
- **Misma salida que `run`:** `load` imprime `Años simulados` (los de **esta** invocación), `Semilla` (la del guardado), `Incremento`, `Pasos` (los del guardado más los nuevos), `Fecha final`, `Eventos`, `Hash de estado` y `Duración real`, con una primera línea `Origen: <archivo> (<fecha del guardado>)`. `--save` agrega al final `Guardado: <archivo>` (en `run` y en `load`; en `load` puede ser el mismo archivo).
- **Continuar es igual a no interrumpir:** `run --years 10 --seed 1 --save a.sav.zst` y luego `load a.sav.zst --years 30` dan el mismo hash que `run --years 40 --seed 1` (con el mismo `--increment`, y también si el `--script` de la primera parte envía sus comandos en esos 10 años; lo cubren los tests de la CLI).
- **`--seed` en `load`:** la semilla viene del guardado, así que pasarla es un error de uso (código 2) en lugar de ignorarla en silencio. `--data-dir` sí aplica: si los datos no son los del guardado, falla con `DataMismatch` (código 1).
- **`--script` en `load`:** no se soporta (error de uso, código 2): sus `at` serían ambiguos (¿desde el inicio de la partida o desde lo cargado?). Los comandos ya enviados quedan en el guardado.
- **Errores (código 1):** archivo inexistente, vacío, que no es zstd, truncado o corrupto, de una versión más nueva, o con datos distintos; el mensaje empieza con `error: no se pudo cargar <archivo>: ...`. Si falla `--save` (p. ej. el directorio no existe), el resumen de la corrida **ya salió por stdout** (la simulación terminó bien) y el error sale por stderr con código 1, sin la línea `Guardado`.

### Scripts de comandos
`run --script <archivo>` envía comandos a la simulación en momentos dados, para repetir una partida exacta. El script es un JSON **versionado**; el del repo es [`examples/script-ejemplo.json`](examples/script-ejemplo.json):

```json
{
  "version": 1,
  "commands": [
    { "at": 0, "command": { "kind": "debug.mark", "args": { "label": "inicio de la corrida" } } },
    { "at": 30, "command": { "kind": "debug.mark", "args": { "label": "un mes después" } } },
    { "at": 30, "command": { "kind": "debug.mark", "args": { "label": "mismo día, se envía después" } } }
  ]
}
```

`cargo run -p heliarca-cli -- run --years 1 --seed 3 --script examples/script-ejemplo.json` agrega a la salida la línea `Script` (después de `Incremento`) y `Comandos aplicados` (después de `Pasos`); sin `--script` esas líneas no aparecen:

```
Años simulados: 1
Semilla: 3
Incremento: d1
Script: examples/script-ejemplo.json (3 comandos)
Pasos: 365
Comandos aplicados: 3
Fecha final: Año 2, día 1, 00:00:00
Eventos: 3
Hash de estado: <32 hex>
Duración real: 0,003 s (121666 pasos/s)
```

**Formato (versión 1).**

- La raíz es un objeto con `version` (hoy sólo `1`; otra versión es un error claro) y `commands` (una lista, que puede ser vacía). Se eligió un objeto y no la lista pelada para poder migrar el formato sin ambigüedad.
- `at`: **días simulados desde el inicio de la corrida**, entero ≥ 0 (`0` es el comienzo; un día son 86 400 s de simulación). Las entradas van **ordenadas por `at`**; las que comparten `at` se envían en el orden del archivo.
- `command.kind` es el nombre del comando y `command.args` un objeto con sus argumentos (opcional si el comando no tiene). Valores admitidos: texto, booleanos y enteros; sin decimales, `null`, listas ni objetos anidados.
- Cualquier campo desconocido es error, en cualquier nivel.

**Comandos disponibles.** Hoy sólo `debug.mark` (`label`: texto de 1 a 64 caracteres, sin caracteres de control), que deja un evento `DebugMark` en el registro. Es **infraestructura de diagnóstico y pruebas, no mecánica del juego**: existe para tener algo real que enviar y reproducir hasta que el diseño del MVP defina los comandos del juego. El catálogo de comandos conocidos es `heliarca_sim::CommandRegistry`, que construye cada comando desde su `kind` y sus argumentos (la misma pareja que guarda el historial, así que el historial se puede reinyectar).

**Cuándo se aplica cada comando.** El bucle avanza la simulación hasta el día `at` y **envía** el comando (`Sim::submit_command`, que lo valida contra el estado); el comando se **aplica al comienzo del paso siguiente**, como siempre. Un `at` debe ser **menor** que los días simulados (`--years` × `days_per_year`): un comando en el último instante nunca llegaría a aplicarse.

**Reproducibilidad.** El mismo script con la misma semilla, años e incremento da el mismo hash; otra semilla, otro script (otro `at`, texto u orden) o el mismo script con otro incremento (el historial guarda en qué paso se aplicó cada comando) dan otro. Un script vacío da el mismo hash que no usar script.

**Errores.** Todo el script se lee, valida y construye **antes** de correr: un error de forma no deja una corrida a medias. Las entradas se numeran **desde 1** (`entrada 3` es la tercera de `commands`) y el mensaje nombra el archivo, la entrada y el campo; sale con código `1` y sin resultados en stdout:

```
$ cargo run -p heliarca-cli -- run --script malo.json
error: script malo.json: entrada 3: campo command.kind: comando desconocido: flota.crear (conocidos: debug.mark)
error: script malo.json: entrada 2: campo at: fuera de orden: 4 es anterior a 5 de la entrada 1
error: script malo.json: entrada 1: campo command.args.extra: argumento desconocido: «extra»
error: script malo.json: entrada 1: campo at: 400 está fuera de la corrida: debe ser menor que 365 (los días simulados)
error: script malo.json: campo version: versión no soportada: 2 (esta CLI sólo entiende la versión 1)
error: script malo.json: JSON malformado en la línea 4, columna 0: EOF while parsing a list
error: script malo.json: no se pudo leer: ... (archivo inexistente, ilegible, que no es UTF-8 o de más de 16 MiB)
```

(Cada línea es un caso distinto; se informa el primer problema.) Hay dos errores que sólo se conocen corriendo: un comando que `validate` rechaza al enviarse y uno que falla al aplicarse (`CommandOutcome::Failed`): ambos detienen la corrida con código `1` y el mensaje indica la entrada (`entrada N (kind): ...`), porque una corrida reproducible no puede seguir en silencio sin un comando del script. `--script` sin valor es un error de uso (código `2`).

### Validar los datos
`cargo run -p heliarca-cli -- validate-data [--data-dir <ruta>]` revisa los archivos de datos (por defecto `data/`) y lista **todos** los errores de una corrida, no sólo el primero. Cada archivo pasa, como mucho, por una de estas fuentes, así que un problema nunca se informa dos veces:

1. **Lectura y sintaxis:** archivo faltante, ilegible, UTF-8 inválido o JSON malformado (`goods.json: JSON malformado en la línea 1, columna 4: ...`). Si un archivo falla acá, no se mira más de él.
2. **Esquema JSON:** el archivo se valida contra el esquema que genera `heliarca_sim::schema` en memoria (no contra `data/schema/`, que el CI ya garantiza que coincide). El error lleva la ruta JSON dentro del archivo (`bodies.json: /1/radius_km: ...`). Se listan hasta 100 errores por archivo.
3. **Modelo del cargador:** sólo para archivos que pasaron el esquema; atrapa lo que éste no expresa (p. ej. `bodies.json: body.x: detect_difficulty: ...`).
4. **Registro de ids y referencias** (formato, duplicados, referencias inexistentes, ciclos de `parent`): sólo si los archivos están sanos, porque con datos incompletos daría falsos «no existe».

Los errores salen por stderr, uno por línea, en el orden de los archivos del cargador; el resumen sale por stdout (`OK: N archivos validados` o `N errores en M archivos`). Códigos de salida: `0` datos válidos, `1` hay errores (o el directorio no existe), `2` error de uso. Ejemplo:

```
$ cargo run -p heliarca-cli -- validate-data --data-dir mis-datos
bodies.json: /1/radius_km: "grande" is not of type "number"
bodies.json: /1: Additional properties are not allowed ('color' was unexpected)
goods.json: JSON malformado en la línea 1, columna 4: key must be a string
recipes.json: falta el archivo
4 errores en 3 archivos
```

Los mensajes de esquema son los del validador (`jsonschema`, en inglés); los del cargador y del registro, en español.

### Esquemas JSON de los datos
Los modelos de `data/` (en `heliarca_sim::data`) derivan `schemars::JsonSchema`, y de ahí sale un esquema JSON (draft 2020-12) por cada archivo de datos, versionado en [`data/schema/`](data/schema/): `data/bodies.json` se valida con `data/schema/bodies.schema.json`, `data/loc/es.json` con `data/schema/loc/es.schema.json`, y así. Sirven para que el editor valide y autocomplete mientras escribís datos (en VS Code, asociá el esquema al archivo con `json.schemas`).

Los esquemas **se generan, no se editan a mano**. Si cambiás un modelo (o el comentario de documentación de un campo, que viaja como `description`), regeneralos y commiteá el resultado:

```
UPDATE_SCHEMAS=1 cargo test -p heliarca-sim --test schemas
```

Un test (`crates/heliarca-sim/tests/schemas.rs`, parte de `cargo test --workspace`) compara los esquemas versionados con los generados y falla, indicando este mismo comando, si no coinciden; así el CI se pone rojo si un modelo cambia sin regenerar. El mismo test valida cada archivo de `data/` contra su esquema y comprueba que el esquema y el cargador (`DataLoader`) coincidan en qué es válido. La salida es determinista (claves ordenadas, LF y salto de línea final; `.gitattributes` fija `eol=lf` en `data/schema/`). El esquema describe la forma (tipos, campos requeridos y desconocidos); las reglas de ids y referencias cruzadas las valida el registro de ids, no el esquema.

### Tests de propiedades
`heliarca-sim` usa [`proptest`](https://crates.io/crates/proptest) (sólo como dependencia de desarrollo) para comprobar propiedades del núcleo con entradas generadas. La central es que **avanzar 30 × `D1` equivale a avanzar 1 × `D30`**, comparando `Sim::state_hash()`; vive en `crates/heliarca-sim/tests/properties.rs` (API pública) y en `crates/heliarca-sim/src/partition_tests.rs` (con sistemas de prueba), y corre con `cargo test --workspace`. Los tests de propiedades son deterministas (semilla fija, 64 casos) y no escriben archivos en el repo. Para explorar más casos o reproducir un fallo: `PROPTEST_RNG_SEED=<u64> PROPTEST_CASES=<n> cargo test -p heliarca-sim`. El detalle (qué entra al hash y cómo agregar una propiedad) está en [CONTRIBUTING.md](CONTRIBUTING.md#tests-de-propiedades).

### Benchmarks
`heliarca-sim` mide el costo del avance con [`criterion`](https://crates.io/crates/criterion) (sólo como dependencia de desarrollo: no entra al árbol de `heliarca-sim`, `heliarca-cli` ni `heliarca-godot`). El código está en `crates/heliarca-sim/benches/advance.rs` y usa sólo la API pública (`Sim::new(Scenario::default(), semilla)` y `Sim::advance`).

**Presupuesto** (doc 21 §1/§2): **1 año de juego en menos de 0,75 s** en modo sin interfaz, que equivale a **40 años en menos de 30 s**. Los benchmarks no hacen fallar nada si se pasan (los tiempos de un runner compartido son ruidosos, así que el CI sólo *compila* los benches, con `cargo clippy --workspace --all-targets`): sirven para detectar regresiones comparando contra una línea base.

**Qué miden.** Avanzar el escenario por defecto, que está **vacío** (sin sistemas, comandos ni eventos): es la **línea base del núcleo** (planificador, tiempo, RNG, registro de eventos), no el juego real. Los sistemas que se vayan sumando agregan costo, y el presupuesto aplica al juego completo, así que un número chico acá no demuestra que el presupuesto se cumpla. La construcción del `Sim` no se cronometra. Benchmarks (nombres estables, no renombrar): `advance_1_year_d1` (365 × `D1`), `advance_1_year_d5` (73 × `D5`), `advance_40_years_d1` (14 600 × `D1`) y `advance_40_years_d5` (2 920 × `D5`). Cada uno tarda unos 3 s (calentamiento y 20 muestras).

**Cómo correrlos.**

```
cargo bench -p heliarca-sim
```

Imprime en consola el tiempo de cada benchmark (`time: [mínimo estimado máximo]`) y deja los reportes en `target/criterion/` (ignorado por git): un HTML por benchmark (`target/criterion/<benchmark>/report/index.html`), el índice `target/criterion/report/index.html` y los datos en JSON (`target/criterion/<benchmark>/new/estimates.json`). `cargo bench` compila en release (perfil `bench`). `cargo test --workspace` NO ejecuta los benchmarks (el CI sólo los compila, por `clippy --all-targets`); para comprobar que no fallan sin medir nada, `cargo test -p heliarca-sim --benches` corre cada uno una sola vez (menos de 1 s en total). Ojo: criterion arrastra una dependencia que compila código C (`alloca`), así que para `cargo bench` y para compilar los benchmarks hace falta un compilador de C (en Windows lo trae el workload de C++ de los Build Tools; en Linux, `build-essential`).

**Cómo comparar contra la línea base.** Criterion guarda cada corrida con un nombre y compara contra la que le pidas. Para medir un cambio:

```
git switch main
cargo bench -p heliarca-sim -- --save-baseline principal   # guarda la línea base «principal»
git switch mi-rama
cargo bench -p heliarca-sim -- --baseline principal        # compara contra «principal»
```

Cada benchmark imprime `change: [-x% ±y% +z%]` y un veredicto (`Performance has improved`, `has regressed` o `No change in performance detected`; ojo: sólo es una regresión real si el intervalo no incluye 0 % y `p < 0.05`). Las líneas base viven en `target/criterion/` (si hacés `cargo clean` se pierden) y valen sólo en la misma máquina, sin otra carga: los tiempos entre máquinas o con la PC ocupada no son comparables, y cambios de ±30 % entre corridas idénticas son comunes en máquinas cargadas. `--save-baseline` también puede reemplazar una línea base existente; para sólo mirar sin comparar, `--load-baseline` y `--baseline` aceptan el nombre de una ya guardada. Un benchmark sin línea base previa (porque es nuevo) se mide igual, sin comparar.

**Números medidos** (Windows 11, AMD Ryzen 7 6800H, Rust 1.100.0, `cargo bench`, con otros procesos de compilación en la máquina, así que son órdenes de magnitud, no valores precisos): `advance_1_year_d1` ≈ 6 µs, `advance_1_year_d5` ≈ 1 a 3 µs, `advance_40_years_d1` ≈ 0,23 ms y `advance_40_years_d5` ≈ 0,05 ms (concuerda con lo medido por la CLI: 40 años en menos de 1 ms en release). Contra el presupuesto, el escenario vacío usa unas 100 000 veces menos tiempo que 0,75 s por año (la diferencia entre el núcleo vacío y el presupuesto es el margen para los sistemas del juego).

### Correr los checks del CI en local
Los scripts de [`scripts/ci/`](scripts/ci/) son los mismos que usa el CI. Necesitan `bash` (en Windows viene con Git for Windows, y se corren desde Git Bash) y se lanzan desde la raíz:

```
for s in scripts/ci/check-*.sh; do bash "$s"; done
```

Cada uno imprime `OK: ...` si pasa y sale con error si no. Protegen, entre otras cosas, que `heliarca-sim` no dependa de Godot ni use `unsafe`, que el toolchain esté fijado y que los alias del README existan.

### Auditoría de dependencias
Con `cargo install --locked cargo-deny` instalado, `cargo deny check` aplica la política de [Dependencias y auditoría](#dependencias-y-auditoría-cargo-deny). Es opcional en local: el CI lo corre igual. Necesita red, porque descarga la base de RustSec.

### Godot
El proyecto está en [`godot/`](godot/) y se abre con **Godot 4.7.2**, la versión fijada (ver [doc 24](Documentacion/24_decision_godot_gdext.md)). Para compilar la extensión no hace falta Godot; sí para abrir el proyecto y probarlo. Este procedimiento se probó en una copia limpia del repo, **sólo en Windows 11** (MSVC, Godot 4.7.2 estándar). En Linux y macOS debería andar igual cambiando el nombre de la biblioteca (ver «Otras plataformas»), pero no se probó.

#### Requisitos
- **Rust 1.100.0** (ver [Requisitos](#requisitos)): `rustup` lo instala solo al entrar al repo.
- **Godot 4.7.2 estándar** (no .NET), de las [compilaciones oficiales](https://github.com/godotengine/godot-builds/releases/tag/4.7.2-stable). Bajá el zip de tu plataforma (`Godot_v4.7.2-stable_win64.exe.zip`, `Godot_v4.7.2-stable_linux.x86_64.zip` o `Godot_v4.7.2-stable_macos.universal.zip`) y el archivo `SHA512-SUMS.txt` de la misma página, y verificá el checksum antes de ejecutar nada:
  - Linux/macOS/Git Bash: `grep win64.exe.zip SHA512-SUMS.txt | sha512sum -c -` (en macOS, `shasum -a 512 -c -`).
  - PowerShell: `(Get-FileHash Godot_v4.7.2-stable_win64.exe.zip -Algorithm SHA512).Hash` y compará (sin importar mayúsculas) con la línea del zip en `SHA512-SUMS.txt`.

  Descomprimí en una carpeta propia: no hay instalador. En Windows quedan `Godot_v4.7.2-stable_win64.exe` (editor) y `Godot_v4.7.2-stable_win64_console.exe` (versión de consola, la que imprime en la terminal; conviene para los comandos de abajo). En lo que sigue, `godot` es ese ejecutable (con su ruta o agregado al `PATH`). Al arrancar, `godot --version` tiene que informar `4.7.2.stable.official...`.

#### Paso a paso
Todo desde la raíz del repo, en una copia recién clonada:

1. **Compilar la extensión:** `cargo build -p heliarca-godot`. Deja `target/debug/heliarca_godot.dll` (Windows). En un clon nuevo compila `godot` (gdext) y el resto desde cero: ~2 min 20 s en la prueba; después es incremental (~2 s).
2. **Importar el proyecto (obligatorio en un clon nuevo):** `godot --headless --path godot --import --quit`. Godot necesita generar `godot/.godot/` (caché de scripts, clases y extensiones; está ignorada por git) y sin ella no conoce las clases de la extensión: correr la escena o el chequeo antes falla con `Identifier "SimHandle" not declared in the current scope` aunque la DLL exista. Este paso lo hace también el editor al abrir el proyecto por primera vez. Es normal ver un `WARNING` sobre `heliarca_godot.pdb` (ver «Qué se ve»). **Ojo con el código de salida:** la primera importación que carga la extensión termina con código `-1073741819` (`0xC0000005`, falla al cerrar Godot) aunque ya generó `godot/.godot/` bien (se comprobó: todo lo que sigue anda); repetir el comando da 0. No encadenes este paso con `&&` ni con `set -e`: ignorá su código o corrélo dos veces.
3. **Abrir el editor:** `godot --editor --path godot`, o abrí la carpeta `godot/` desde el Project Manager. La consola tiene que mostrar `Initialize godot-rust (API v4.7...)` y `Heliarca: extensión cargada (heliarca-sim 0.1.0)`, sin errores. Al cerrarlo, el editor reescribe el encabezado de comentarios de `godot/project.godot`: no lo commitees (`git checkout godot/project.godot`).
4. **Correr la escena sin abrir el editor:** `godot --headless --path godot --quit-after 10`, que imprime `Heliarca (sim 0.1.0): Año 1, día 1, 00:00:00`.
5. **Chequeo automático:** `godot --headless --path godot --script res://tests/check_main.gd` imprime `OK: la etiqueta muestra '...'` y sale con código 0; con código 1 si algo no coincide (ver «La escena `Main`»).

Para borrar el estado de Godot y repetir desde cero basta borrar `godot/.godot/` (y volver al paso 2).

#### Qué se ve
- **Falta la DLL** (no se corrió el paso 1 o se hizo `cargo clean`): el editor imprime `GDExtension dynamic library not found: 'res://heliarca.gdextension'` y `Error loading extension`, y abre igual el proyecto sin la extensión. Compilá y reabrí. La escena y el chequeo fallan con `SimHandle` no declarado.
- **`WARNING: The original path size of 'heliarca_godot.pdb' ... was shortened to '~heliarca__999.pdb'`** (`copy_and_rename_pdb`): Godot copia la DLL y su `.pdb` con otro nombre para poder recargarla (ver «Recarga en caliente»). Es inocuo.
- `godot/heliarca.gdextension` apunta directo al `target/` del workspace (`res://../target/{debug,release}/...`), así que no hay que copiar nada. El editor usa la entrada `debug` (`cargo build`); la `release` es para el juego exportado (`cargo build --release`). `res://../` sale de la carpeta del proyecto, así que **no sirve para exportar**: al exportar hay que copiar la biblioteca dentro de `godot/` y apuntar el `.gdextension` ahí (queda para la tarea de exportación).
- El `heliarca.gdextension.uid` sí se versiona (recomendación oficial de Godot 4.4+).

#### Otras plataformas
El `.gdextension` ya declara las rutas de Linux (`target/debug/libheliarca_godot.so`) y macOS (`libheliarca_godot.dylib`), con `compatibility_minimum = "4.7"` y `entry_symbol = "gdext_rust_init"`. En Linux hace falta `build-essential` (ver [Requisitos](#requisitos)). El CI compila el `cdylib` en Ubuntu y Windows con [`scripts/build-godot-cdylib.sh`](scripts/build-godot-cdylib.sh) (también se puede correr en local; falla si no compila o no genera la biblioteca que declara el `.gdextension`), pero **no abre Godot**: en Linux y macOS ni la apertura del editor ni el recargado en caliente están probados.

#### La escena `Main`
`Main.tscn` tiene una `Label` y el script [`godot/Main.gd`](godot/Main.gd): en `_ready` crea un `SimHandle` (clase gdext de `heliarca-godot`, base `RefCounted`, que posee un `Sim`), llama a `hello()` y muestra el texto en la etiqueta y en la consola. El texto lo arma `heliarca-sim` (versión del núcleo y fecha inicial), por ejemplo `Heliarca (sim 0.1.0): Año 1, día 1, 00:00:00`. `SimHandle` no tiene lógica de juego: sólo posee el `Sim` y envuelve su API pública (`set_seed(semilla)` lo reinicia con otra semilla).

`godot/tests/check_main.gd` carga la escena, espera un frame y comprueba que la etiqueta coincide con `SimHandle.hello()` **y** con el valor esperado de forma independiente: `Heliarca (sim <versión del Cargo.toml raíz>): Año 1, día 1, 00:00:00`. Así un cambio en la cola de `hello_text` (versión o fecha) hace fallar el chequeo aunque la etiqueta siga igualando a `hello()`. Si no hay extensión compilada, falla con `SimHandle` no declarado.

#### Recarga en caliente (probada en Windows)
`heliarca.gdextension` tiene `reloadable = true`. Se probó con Godot 4.7.2 y gdext 0.5.5 en Windows 11, con el editor abierto sobre `godot/` y un `SimHandle` vivo, recompilando con `cargo build -p heliarca-godot` (detalle y evidencia en el [doc 24](Documentacion/24_decision_godot_gdext.md), §2).

**Resultado: funciona, con límites.**
- **Recompilar con el editor abierto anda en Windows.** Godot no bloquea `target/debug/heliarca_godot.dll`: carga una copia (`~heliarca_godot.dll`, con su `~heliarca__0.pdb`, en la misma carpeta `target/debug/`) y `cargo build` reemplaza el original sin errores.
- **El editor recarga solo, pero al recuperar el foco la ventana**, no apenas termina la compilación: con la ventana ya enfocada no se recargó en más de un minuto; al salir y volver a ella la consola volvió a imprimir `Initialize godot-rust ...` y `Heliarca: extensión cargada ...`, y `SimHandle.new().hello()` pasó a devolver el texto nuevo sin reiniciar el editor. O sea: compilá, cambiá de ventana y volvé al editor.
- **Cambios de código soportados:** cuerpos de funciones (texto de `hello_text`), agregar campos a la clase, agregar métodos `#[func]`, cambiar el tipo de retorno y agregar una clase nueva (aparece en `ClassDB`).
- **Se pierde el estado de Rust:** las instancias vivas siguen siendo objetos válidos y pasan a usar el código nuevo, pero sus campos Rust se reinician a los valores de `init` (un `SimHandle` con `set_seed(7)` o un `Sim` avanzado vuelve a un `Sim` nuevo). Los valores guardados del lado de GDScript (referencias, variables) no se tocan.
- **Cambios de firma:** las llamadas ya escritas en GDScript contra la firma vieja fallan con error de script al reejecutarse (por ejemplo `hello()` sin el argumento nuevo); hay que actualizar los llamadores.
- **Una escena corriendo (F5, o `godot --path godot` en otra ventana) NO se recarga y bloquea la DLL**: es otro proceso que la tiene abierta y `cargo build` falla con `failed to remove file ...heliarca_godot.dll: Access is denied. (os error 5)`. Hay que cerrar la escena antes de recompilar.
- Dentro del editor, una instancia creada con `ClassDB.instantiate("Clase")` de una clase que no es `tool` es un placeholder y llamarle métodos da error (`Cannot call GDExtension method bind ... on placeholder instance`); `SimHandle.new()` desde un script `@tool` sí ejecuta el código. No afecta al juego en ejecución. Quitar métodos o campos y la escena corriendo dentro del editor no se probaron.

**Plan B** (si el editor se cuelga, la DLL queda bloqueada o hay una escena abierta): cerrar el editor y la escena, `cargo build -p heliarca-godot` y reabrir. Si un día el editor bloqueara la DLL, copiar la compilación con otro nombre (por ejemplo `godot/bin/heliarca_godot_dev.dll`) y apuntar el `.gdextension` a esa copia.

## Cómo se trabaja
El desarrollo está planificado en **issues de GitHub** con jerarquía `Hito → Épica → Historia → Tarea`. Leé [CONTRIBUTING.md](CONTRIBUTING.md) y el [plan de desarrollo](Documentacion/23_plan_de_desarrollo.md). Todo cambio entra por **Pull Request**; `main` está protegida (ver [SECURITY.md](SECURITY.md)).

## Tiempos de compilación
Medidos el 2026-11-17 en un AMD Ryzen 7 6800H (8 núcleos / 16 hilos, 15 GB de RAM), Windows 11 con el target `x86_64-pc-windows-msvc` y Rust 1.100.0, con `cargo build --workspace` (perfil `dev`, linker `link.exe` de MSVC). Son 6 corridas cada una:

| Caso | Tiempo |
|---|---|
| Compilación completa (`cargo clean` + `cargo build --workspace`) | ~0,46 s (0,45–0,51 s) |
| Incremental: cambio trivial en `heliarca-sim` (recompila también `heliarca-cli` y `heliarca-godot`, que dependen de él) | ~0,5 s (0,47–0,64 s) |
| Incremental: cambio trivial en `heliarca-cli` (solo recompila `heliarca-cli`) | ~0,3 s |

Son valores bajísimos porque los crates todavía son casi cáscaras vacías y no hay dependencias externas: sirven de línea base para comparar cuando el código crezca, no como promesa de rendimiento. Un cambio trivial recompila solo el crate tocado y sus dependientes: tocar `heliarca-cli` no recompila `heliarca-sim` ni `heliarca-godot` (Cargo los marca como `Fresh`).

Qué trae la configuración de [`.cargo/config.toml`](.cargo/config.toml) y del `Cargo.toml` raíz:

- **Alias:** `cargo c` (`check` de todo el workspace), `cargo t` (`test` de todo el workspace) y `cargo l` (el mismo `clippy -D warnings` del CI).
- **Dependencias optimizadas:** `[profile.dev.package."*"] opt-level = 3` compila las dependencias externas con optimizaciones aunque el código propio vaya en debug. Hoy el workspace no tiene dependencias externas, así que **todavía no tiene efecto medible**; está puesto para cuando entren `godot` y otras.
- **Linker rápido:** no está activado. `rust-lld` ya viene con el toolchain y lo probé (`CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=rust-lld.exe`), pero con los crates actuales no mejora nada (~0,47 s completa y ~0,5 s incremental, igual que `link.exe`), y `config.toml` no puede activarlo "solo si está instalado" sin romper el CI o a quien no lo tenga. Cómo probarlo localmente está comentado en el propio `config.toml`; conviene volver a medir cuando haya dependencias pesadas.

## Dependencias y auditoría (cargo-deny)

[`deny.toml`](deny.toml) define la política: licencias permisivas (MIT, Apache-2.0, BSD, ISC, Unicode-3.0, Zlib), fuentes solo de crates.io, versiones duplicadas en aviso y avisos de RustSec (o crates yanked) como error. El job `build-test` de Linux corre `cargo-deny check` con un binario oficial fijado por versión y SHA-256. Localmente: `cargo install --locked cargo-deny` y `cargo deny check`. `cargo-audit` no se usa: usa la misma base de RustSec y `cargo-deny` ya la cubre.

El CI baja la base de RustSec en cada corrida, así que un aviso nuevo sobre una dependencia que ya teníamos puede poner rojo el CI sin que cambie nada en el repo. Es a propósito; se arregla actualizando la dependencia o, si no hay arreglo, con una entrada en `advisories.ignore` de `deny.toml` con id y motivo, revisada en un PR.
