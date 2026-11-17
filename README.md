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

- `run`: avanza la simulación N años e imprime fecha final, eventos, hash de estado y duración real (ver [Correr la simulación](#correr-la-simulación)).
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
Hash de estado: 20be5da18cee18825dbc5e2a0ab3e06c
Duración real: 0,002 s
```

Opciones de `run` (además de `--seed` y `--data-dir`):

- `--years <N>`: años a simular, de 0 a 10000 (por defecto `40`, el horizonte del criterio de rendimiento). Un año son 365 días, el del calendario que muestra la fecha (ver el issue #439 sobre `days_per_year`). `--years 0` es válido: 0 pasos. Fuera de rango es un error de uso (código 2).
- `--increment <s5|h1|d1|d5|d30>`: incremento de cada paso (por defecto `d1`). Si lo que falta es menos que el incremento, el último tramo se completa con incrementos menores (D30, D5, D1, H1, S5), así que nunca se pasa del total. Si el núcleo interrumpe un avance, la CLI sigue desde donde quedó; si un avance no progresa nada, corta con error (código 1) en vez de quedarse girando.

Los datos se cargan con la misma política completa que `validate-data` (archivos más registro de ids y referencias); si fallan, los errores salen por stderr, uno por línea, con código 1. Las líneas de la salida son estables y parseables (`Clave: valor`); todas salvo `Duración real` son deterministas: misma semilla, años e incremento dan el mismo hash. El hash incluye el estado interno (semilla/RNG, pasos), así que cambiar el incremento puede cambiar el hash aunque la fecha final sea la misma.

**Duración real y línea base de rendimiento.** Es el tiempo de pared del bucle de avance (sin la carga de datos), con tres decimales y coma decimal (`0,002 s`). Se mide sólo en la CLI (`crates/heliarca-cli/src/timing.rs`) y no entra a la simulación ni al hash. Es la línea base para el criterio «40 años de juego en menos de 30 s» (doc 21 §1). Con el escenario actual (vacío, sin sistemas), 40 años a `d1` tardan en una máquina de desarrollo con Windows 11: 0,002 s en debug (`cargo run -p heliarca-cli -- run --years 40 --seed 1`) y menos de 1 ms en release (`cargo run --release -p heliarca-cli -- run --years 40 --seed 1`); a `s5` (252 millones de pasos en 40 años) tarda unos 3,9 s en release. Son números de referencia que irán subiendo cuando existan los sistemas; el presupuesto se vigila contra el criterio de 30 s, no contra estos valores.

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

### Correr los checks del CI en local
Los scripts de [`scripts/ci/`](scripts/ci/) son los mismos que usa el CI. Necesitan `bash` (en Windows viene con Git for Windows, y se corren desde Git Bash) y se lanzan desde la raíz:

```
for s in scripts/ci/check-*.sh; do bash "$s"; done
```

Cada uno imprime `OK: ...` si pasa y sale con error si no. Protegen, entre otras cosas, que `heliarca-sim` no dependa de Godot ni use `unsafe`, que el toolchain esté fijado y que los alias del README existan.

### Auditoría de dependencias
Con `cargo install --locked cargo-deny` instalado, `cargo deny check` aplica la política de [Dependencias y auditoría](#dependencias-y-auditoría-cargo-deny). Es opcional en local: el CI lo corre igual. Necesita red, porque descarga la base de RustSec.

### Godot
> **Pendiente:** se completa en M0.7 (instalación de Godot 4, compilación de `heliarca-godot` y cómo abrir el proyecto). Por ahora `cargo build --workspace` compila `heliarca-godot` como un crate común, sin Godot instalado.

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
