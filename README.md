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
