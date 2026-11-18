# 24 — Decisión: versiones de Godot 4 y gdext

> v0.1 · Fecha de la decisión: **2026-11-17** (verificada contra fuentes de noviembre de 2026). Spike **M0.7.1.1** (#57). Cierra la parte de «versión» de la pregunta abierta nº 38 de 17; la parte de «recarga en caliente» se confirma con la práctica en #60 (M0.7.1.4). Contexto de arquitectura: 21 §5 (`heliarca-godot` es un `cdylib` fino, sin lógica de juego).

## 1. Decisión

| Pieza | Versión fijada | Dónde |
|---|---|---|
| **Godot** (editor y runtime) | **4.7.2-stable** (parche más reciente de la rama 4.7; 4.7-stable salió el 2026-06-18, 4.7.2 el 2026-08-18) | proyecto `godot/` (M0.7.1.2) y guía del README |
| **gdext** (crate `godot`) | **`=0.5.5`** (publicada el 2026-08-09) | `[workspace.dependencies]` de `Cargo.toml` |
| Feature de API | **`api-4-7`** (API de GDExtension de Godot 4.7.0, corre en cualquier 4.7.x) | idem |
| Rust | **1.100.0** (el fijado en `rust-toolchain.toml`) | MSRV de gdext 0.5.5: **1.94**, así que hay margen |

En `Cargo.toml`:

```toml
[workspace.dependencies]
godot = { version = "=0.5.5", features = ["api-4-7"] }
```

Se usan las features por defecto de `godot` (`__codegen-full`, que genera todas las clases de Godot), más `api-4-7`. **Todavía no la usa ningún crate**: `heliarca-godot` la adopta en #58/#59 con `godot = { workspace = true }`. Declararla en el workspace sin usarla no cambia `Cargo.lock` ni el grafo de `heliarca-sim`/`heliarca-cli`/`heliarca-godot` (verificado: `cargo build --workspace --locked` y `cargo deny check` pasan igual).

### Por qué estas versiones

- **Godot 4.7.2 es la última estable** de la serie 4.x (GitHub Releases de `godotengine/godot`, 2026-11). Las ramas 4.6 y 4.5 siguen con parches, pero arrancar un proyecto nuevo en una rama a punto de quedar atrás no aporta nada.
- **gdext 0.5.5 es la última publicada** en crates.io y la primera línea con nivel de API 4.7 completo: `api-4-7` llegó en 0.5.4 (2026-06-23) y 0.5.5 es el parche siguiente. El changelog de 0.5.2 y 0.5.3 ya trae ajustes de la inicialización y de la nulabilidad de 4.7.
- **Compatibilidad hacia adelante:** una extensión compilada contra API 4.7 corre en runtime **>= 4.7** (nunca en uno anterior). Es lo que se necesita: el editor y la build del juego son 4.7.x, así que `api-4-7` es el techo natural sin perder nada de la API.
- **Versión exacta (`=`):** gdext sigue semver pero el API de Rust cambia entre menores (0.x) y la API de Godot sube cada pocos meses. Subir de versión será una decisión explícita en un PR propio (ver §6).
- **Sin `api-custom`:** esa feature genera la API a partir de un binario local de Godot (`GODOT4_BIN`) y exige LLVM; no hay garantías de compatibilidad. Con `api-4-7` no hace falta ningún binario de Godot para compilar.

## 2. Compatibilidad verificada

- **Rust:** `godot` 0.5.5 declara `rust-version = 1.94` y edición 2024; el workspace usa 1.100.0 y edición 2024. Compatible.
- **Plataformas:** Windows, Linux y macOS son las de escritorio soportadas (el CI del repo ya cubre Ubuntu y Windows; macOS queda para cuando se haga el *export*). Android, iOS y Wasm son experimentales en gdext y no aplican al MVP.
- **cdylib en Windows MSVC:** sin problemas en la prueba (ver §3): el `cdylib` se enlaza y genera la DLL con el toolchain MSVC por defecto, sin LLVM ni binario de Godot. Es el mismo tipo de crate que ya tiene `heliarca-godot`.
- **Mínimo de Godot:** gdext 0.5.x exige Godot >= 4.2; no es una restricción para nosotros.
- **Recarga en caliente (relevante para #60):** gdext tiene soporte y lo prueba en su CI (ejemplo `hot-reload`); sus parches recientes arreglan pánicos al recargar (0.5.4 en Linux, 0.5.5 con `OnEditor<Gd<T>>`). Requiere `reloadable = true` en el `.gdextension` y funciona desde Godot 4.2. **Incertidumbre en Windows:** históricamente el editor bloquea la DLL abierta y no deja recompilar con el editor abierto (issue godotengine/godot#66231, cerrado en 4.2 con la recarga de GDExtension, pero sin confirmación explícita del bloqueo de archivo). **Se confirma en #60** con una prueba real en Windows; plan B documentado: cerrar el proyecto antes de recompilar, o copiar la DLL con otro nombre (`heliarca_godot_dev.dll`) hacia `godot/bin` y apuntar el `.gdextension` a esa copia.

## 3. Prueba de compilación (Windows, toolchain 1.100.0, sin Godot instalado)

Crate temporal **fuera del repo** con `godot = { version = "=0.5.5", features = ["api-4-7"] }`, `crate-type = ["cdylib"]`, `#[gdextension]` y una clase `#[derive(GodotClass)]` con un `#[func]` (equivalente a `SimHandle::hello()` de #59). Sin `godot`, sin `GODOT4_BIN` y sin LLVM instalados:

| Caso | Resultado |
|---|---|
| `cargo build` (debug), limpio | OK, ~60 s (incluye compilar todas las dependencias, con codegen completo) |
| `cargo build --release`, limpio | OK, ~80 s |
| Reconstrucción incremental (cambio en el crate propio) | ~1,5 s |
| DLL generada | `spike.dll` ~2,9 MB (debug) / ~3,8 MB (release) |

Un primer intento con `default-features = false` falló al derivar `GodotClass` sobre `RefCounted`, porque sin `__codegen-full` no se generan las clases del motor: por eso se mantienen las features por defecto. No hubo descargas de `extension_api.json` en la compilación (el crate `gdextension-api` ya lo trae empaquetado, v0.5.1).

Efecto sobre el repo: `[profile.dev.package."*"] opt-level = 3` ya compila las dependencias externas optimizadas aunque el código propio vaya en debug, así que esos tiempos son los de una compilación limpia de `godot`; el día a día es el incremental.

## 4. Licencias

`godot` y sus crates hijos son **MPL-2.0** (copyleft *por archivo*: sólo obliga a publicar modificaciones a los archivos MPL de gdext; el código propio puede seguir siendo propietario y enlazarse con ellos). **El dueño del repo ya decidió permitir MPL-2.0** en `deny.toml`; **se agregó a `deny.toml` en #58** (junto con la primera dependencia real).

Licencias de todo el grafo (`cargo metadata` del crate temporal, con `godot 0.5.5` + `api-4-7`):

| Crate | Licencia |
|---|---|
| `godot`, `godot-core`, `godot-ffi`, `godot-codegen`, `godot-macros`, `godot-bindings`, `godot-cell`, `gdextension-api` | MPL-2.0 |
| `glam`, `heck`, `libc`, `nanoserde`, `proc-macro2`, `quote` | MIT OR Apache-2.0 |
| `nanoserde-derive`, `venial` | MIT |
| `unicode-ident` | (MIT OR Apache-2.0) AND Unicode-3.0 |

Corrida de `cargo deny check licenses` con el `deny.toml` actual sobre ese crate: **8 rechazos, todos MPL-2.0** (los 8 crates de la primera fila). **No aparece ninguna otra licencia nueva**: el resto ya está cubierto por la lista (MIT, Apache-2.0, Unicode-3.0). `nanoserde`, `nanoserde-derive`, `heck` y `venial` son dependencias de compilación/macros y no llegan al binario. `Cargo.lock`, en cambio, sí incluirá todos esos crates cuando `heliarca-godot` la use.

Hecho en #58: `"MPL-2.0"` está en `[licenses] allow` de `deny.toml` con el comentario del motivo. No se agregó ninguna otra licencia.

## 5. Alternativas descartadas

- **Godot 4.6.x / 4.5.x:** válidas y con parches, pero más viejas sin ventaja; gdext 0.5.5 las soporta igual (`api-4-6`, `api-4-5`) si hiciera falta bajar el piso de compatibilidad.
- **gdext 0.4.x:** línea anterior, sin `api-4-7` ni edición 2024.
- **gdext desde `git` (`master`):** `deny.toml` prohíbe dependencias por git, y la rama principal puede romper la API sin aviso.
- **`api-custom` / `api-custom-json`:** exigen un binario de Godot al compilar (o un JSON propio) y no tienen garantía de compatibilidad; complicarían el CI sin necesidad.
- **Binding C#/.NET de Godot:** descartado antes (17, decisión 55: núcleo en Rust).
- **Compilar contra `api-4-x` más viejo para ampliar compatibilidad:** innecesario, porque distribuimos nosotros el editor/runtime 4.7.x.

## 6. Riesgos

| Riesgo | Mitigación |
|---|---|
| API de gdext cambiante (0.x) | Versión exacta `=0.5.5`; puente fino (21 §5); subir versión en PR propio leyendo la guía de migración |
| Cada versión menor de Godot pide su `api-4-x` | Al subir de Godot, cambiar la feature y la versión de gdext juntas; el CI de #61 detecta roturas |
| Descargas / red al compilar | Con `api-4-7` no hay descargas (el JSON viene dentro del crate `gdextension-api`); `api-custom` queda prohibida |
| Necesidad de un binario de Godot para compilar | **No hace falta** con `api-4-7`. Sí hace falta para abrir el proyecto y probarlo (instalar 4.7.2) |
| Tiempos de compilación | ~60–80 s limpio; incremental ~1,5 s; el caché de CI (#61) absorbe el costo. `opt-level = 3` de dependencias ya está |
| Recarga en caliente en Windows | Ver §2; se valida en #60 con plan B |
| `unsafe` en el FFI | `heliarca-godot` hereda `unsafe_code = "deny"`; `#[gdextension]` exige `unsafe impl`, así que se relaja con `#![allow(unsafe_code)]` sólo en el módulo `extension` de ese crate (21 §5; en #58 se comprobó que un `#[allow]` sobre el `impl` no alcanza, la expansión de `#[gdextension]` lo ignora). `heliarca-sim` sigue con `forbid` |
| Licencia MPL-2.0 | Ya aprobada por el dueño (§4) |

## 7. Pasos siguientes

- **#58 (M0.7.1.2) — Proyecto Godot con `.gdextension`:** crear `godot/` con `project.godot` para 4.7 y el `.gdextension` (rutas a la DLL/so/dylib de debug y release, `compatibility_minimum = "4.7"`, `reloadable = true`; hecho en #58, con rutas directas a `../target` y sin `godot/bin/`). Hacer que `heliarca-godot` dependa de `godot = { workspace = true }`, con `[lints] workspace = true` y el `#[allow(unsafe_code)]` mínimo; agregar `MPL-2.0` a `deny.toml`; `check-sim-isolation.sh` sigue exigiendo que `heliarca-sim` no tenga `godot`.
- **#59 (M0.7.1.3) — `SimHandle` con `hello()`:** clase `#[derive(GodotClass)]` que sólo delega en `heliarca-sim` (p. ej. `sim_version()`); probarla desde GDScript en Godot 4.7.2.
- **#60 (M0.7.1.4) — Recarga en caliente y guía:** probar recompilar con el editor abierto en Windows; documentar el procedimiento y el plan B; completar la sección «Godot» del README con la instalación de 4.7.2 y `cargo build -p heliarca-godot`.
- **#61 (M0.7.1.5) — Compilar el `cdylib` en CI:** ya está en la matriz Ubuntu y Windows (`cargo build --workspace --locked`); queda agregar caché de dependencias y, si se quiere, un chequeo de que la versión de `godot` en `Cargo.lock` es `0.5.5` y la de Godot documentada coincide.
- Cuando salga Godot 4.8: evaluar subir (nueva entrada en este documento), no actualizar «en silencio».

## 8. Fuentes consultadas (2026-11-17)

- crates.io: `cargo info godot` → 0.5.5, MPL-2.0, `rust-version 1.94`, features `api-4-2` … `api-4-7`, `api-custom`.
- [Changelog de gdext](https://github.com/godot-rust/gdext/blob/master/Changelog.md): 0.5.5 (2026-08-09), 0.5.4 (2026-06-23, API 4.7), 0.5.3, 0.5.0 (edición 2024, API 4.6).
- [The godot-rust book — Compatibilidad](https://godot-rust.github.io/book/toolchain/compatibility.html) y [Selección de versión de Godot](https://godot-rust.github.io/book/toolchain/godot-version.html).
- GitHub Releases de `godotengine/godot`: 4.7.2-stable (2026-08-18), 4.7.1 (2026-07-14), 4.7 (2026-06-18).
- Issue [godotengine/godot#66231](https://github.com/godotengine/godot/issues/66231) (recarga de GDExtension y bloqueo de DLL en Windows).
