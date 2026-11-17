# Cómo trabajar en el proyecto

## Regla de oro
**Nada se sube directo a `main`.** Todo pasa por Pull Request. La regla aplica también al dueño del repo.

## Flujo
1. Elegí una **tarea** (`type:task`) del hito actual cuyas dependencias estén cerradas (ver *Orden* en el [plan](Documentacion/23_plan_de_desarrollo.md)).
2. Creá una rama desde `main` actualizada: `tipo/M{hito}-{id}-descripcion-corta`, p. ej. `feat/m0-1-1-1-workspace`.
   Tipos: `feat`, `fix`, `docs`, `test`, `chore`, `refactor`, `perf`.
3. Cambios **chicos**: un PR = una tarea. Si crece más de ~1 día, dividila (abrí issues nuevos bajo la misma historia).
4. Commits en imperativo, con prefijo de tipo: `feat(sim): agrega planificador multi-frecuencia`.
5. Abrí el PR con la plantilla y `Closes #N`. El check **`validate`** debe estar en verde y las conversaciones resueltas.
6. Se integra con **squash merge** (historial lineal). La rama se borra sola.

> Revisión: al ser un proyecto de una persona, la regla **no exige aprobaciones** (nadie puede aprobar su propio PR), pero sí PR + CI + conversaciones resueltas. Si se suman colaboradores, subir `required_approving_review_count` a 1 en el ruleset *Proteger main*.

## Jerarquía de issues
```
Hito (GitHub Milestone: M0…M8)
 └─ Épica   (type:epic)    una funcionalidad grande y cerrable
     └─ Historia (type:story)  valor entregable; "Como… quiero… para…"
         └─ Tarea (type:task)   ≤ ~1 día; requisitos + criterios de aceptación
```
Los issues usan **sub-issues** de GitHub (el progreso de cada padre se calcula solo). Los códigos del título siguen el árbol: `M2.3.1.4` = hito 2, épica 3, historia 1, tarea 4. Las **dependencias** se listan en cada tarea ("Bloqueada por #N") y están ordenadas: **ninguna tarea depende de otra de número mayor**.

Otros tipos: `type:spike` (investigación acotada que termina en una decisión) y `type:decision` (decisión de diseño pendiente que bloquea trabajo).

## Etiquetas
| Grupo | Valores |
|---|---|
| Tipo | `type:epic` · `type:story` · `type:task` · `type:spike` · `type:decision` |
| Tamaño | `size:S` (≤ ½ día) · `size:M` (~1 día) · `size:L` (~2 días; si crece, dividir) |
| Área | `area:infra` `core` `data` `world` `research` `economy` `logistics` `design` `society` `companies` `programs` `cli` `godot` `ui` `testing` `balance` `docs` |
| Estado | `status:needs-breakdown` (falta dividir en tareas) · `status:optional` (recortable) |

## Definición de listo (para empezar una tarea)
- Tiene **requisitos** y **criterios de aceptación** verificables.
- Sus dependencias están cerradas.
- Sabés qué docs del GDD aplica (están enlazados en el issue).

## Definición de hecho (para cerrarla)
- Cumple **todos** los criterios de aceptación.
- Tiene pruebas (unitarias, de propiedad con `proptest` o de escenario); **sin pruebas no se acepta** (doc 21 §4).
- `cargo fmt --check`, `cargo clippy -- -D warnings` y `cargo test` pasan.
- Todo número del GDD vive en **datos/parámetros**, no hardcodeado.
- La simulación sigue siendo **determinista** (misma semilla + comandos ⇒ mismo resultado) y respeta los **invariantes** (no se crean ni destruyen recursos sin receta).
- Si la implementación cambia el diseño, se actualiza el doc y se registra la decisión en `17`.

## Principios técnicos (doc 21 §2)
1. Simulación separada del render: `heliarca-sim` no depende de Godot.
2. Determinista: RNG con semilla, orden estable (`BTreeMap`/`IndexMap`), `f64`, sin tiempo real.
   - Única excepción: la CLI puede medir tiempo real (la «Duración real» de `run`) **sólo** en `crates/heliarca-cli/src/timing.rs`, con un `allow` de clippy acotado a ese módulo. Nunca se usa dentro de `heliarca-sim` ni entra al estado ni al hash.
   - En `heliarca-sim` están prohibidos `HashMap` y `HashSet` (y `RandomState`/`DefaultHasher`): su orden de iteración cambia entre ejecuciones y rompería el determinismo. Usar `BTreeMap`/`BTreeSet`, o `IndexMap`/`IndexSet` (crate `indexmap`, a agregar cuando haga falta).
   - Se hace cumplir con `clippy::disallowed_types` en `crates/heliarca-sim/clippy.toml`; el CI corre `cargo clippy ... -D warnings` y falla si aparecen, también en tests. `heliarca-cli` y `heliarca-godot` no están alcanzados.
   - Clippy no mira dependencias: un tipo de otra librería con hasher aleatorio interno no se detecta, hay que revisarlo en code review.
3. Dirigida por datos: bienes, recetas, techs, diseños, cuerpos y textos en JSON.
   - **Regla: todo número del GDD es un parámetro.** Lo que el diseño pueda querer ajustar (o que el GDD fije como convención: días por semana, mes y año, doc 22 §0) vive en `data/params.json` y se lee vía `Sim::params()`; nunca como literal en el código. Cambiar el JSON cambia el comportamiento sin recompilar.
   - Quedan como constantes de código sólo las **definiciones de unidad** o identidades físicas que ningún diseño ajusta (60 s por minuto, 60 min por hora, 24 h por día). Ante la duda: si el diseño podría querer otro valor, es un parámetro.
   - Para agregar un parámetro: campo en `Params` (con la unidad en el nombre) + valor por defecto del GDD + validación de rango + entrada en `data/params.json` (todos son obligatorios; un campo faltante o desconocido es error) + esquema JSON (se deriva de la misma lista de campos y de `MAX_DAYS`; regenerarlo como se indica abajo) + tests. Detalle en el módulo `heliarca_sim::params`.
   - Los modelos de `heliarca_sim::data` derivan `JsonSchema` y los esquemas viven en `data/schema/`. Si tocás un modelo (o la documentación de sus campos), regenerá los esquemas con `UPDATE_SCHEMAS=1 cargo test -p heliarca-sim --test schemas` y commiteá el resultado: el test `tests/schemas.rs` falla en el CI si quedan desactualizados, y también valida `data/` contra ellos.
4. El jugador actúa por **comandos**; la UI lee **modelos de vista**, no el estado interno.
5. IDs tipados y arenas; errores con `Result`; `clippy` y `rustfmt` obligatorios.

## Tests de propiedades
Los tests de propiedades usan `proptest` (dev-dependency de `heliarca-sim`, con `default-features = false` y sólo `std`; no entra al código de producción ni a `heliarca-godot`). Hay dos niveles:

- `crates/heliarca-sim/tests/properties.rs`: integración, sólo con la **API pública** (`Sim`, `Scenario`...). Hoy es el caso «escenario vacío», porque desde afuera no se pueden registrar sistemas.
- `crates/heliarca-sim/src/partition_tests.rs`: dentro del crate, con **sistemas de prueba** periódicos que emiten eventos y consumen un subflujo del RNG.

La propiedad de ejemplo (#45): **30 × `D1` ≡ 1 × `D30`**, y en general cualquier partición del mismo total de días, comparando `Sim::state_hash()`. Se enuncia **sin comandos y sin sistemas `EveryIncrement`**, que dependen de la partición por diseño.

### Qué es «estado» y qué es «contabilidad de llamadas»
El hash de estado cubre el **estado simulado** y el **historial de entradas** (los comandos), no *cómo se partió el avance*. Por eso `Sim::steps()` (cantidad de llamadas a `advance`) es observable pero **no** entra al hash. En cambio `CommandRecord::step` (en qué llamada se aplicó un comando) sí entra: es parte de las entradas del jugador. Un sistema `EveryIncrement` corre una vez por llamada, así que su efecto también depende de la partición. Cambiar qué entra al hash es un cambio de formato: se sube `FORMAT_VERSION`, se regeneran los hashes «golden» y se avisa en el PR que invalida repeticiones previas (ver `state_hash.rs`).

### Agregar una propiedad
1. Escribila dentro de `proptest! { #![proptest_config(common::config())] ... }` (en `properties.rs`, con `mod common;`) o con `crate::proptest_config::config()` (en el crate). La configuración común está en `crates/heliarca-sim/tests/common/mod.rs`.
2. Usá `prop_assert!`/`prop_assert_eq!` (no `assert!`) para que proptest pueda reducir el caso. Sin `HashMap`/`HashSet`, `std::time` ni `unsafe`, como en el resto de `heliarca-sim`.
3. Mantené el costo bajo (el CI corre en Linux y Windows): la configuración usa 64 casos por propiedad.

### Reproducir un fallo
La configuración es **determinista por defecto**: semilla fija y los mismos casos en cada corrida (el CI nunca es intermitente), y **no persiste fallos** (no se crea `proptest-regressions/`). Si una propiedad falla, proptest imprime el caso mínimo (`minimal failing input`): copialo a un test normal para reproducirlo y dejarlo como regresión. Para explorar con otras entradas:

```
PROPTEST_RNG_SEED=12345 PROPTEST_CASES=2000 cargo test -p heliarca-sim --test properties
```

`PROPTEST_RNG_SEED` (un `u64`) fija la semilla; `PROPTEST_CASES`, la cantidad de casos; el resto de las variables `PROPTEST_*` también se respetan. Cuando una corrida con otra semilla falla, esa semilla más el caso mínimo son suficientes para reproducirlo.

## Invariantes del simulador
Un **invariante** es una regla que el estado de `Sim` cumple siempre entre dos pasos (el tiempo no retrocede, los ids de evento son secuenciales, cada causa es anterior a su efecto, los ids de comando crecen...). Es la **red de seguridad** del doc 21 §2 y §4 («no se crean ni destruyen recursos sin receta» será uno más cuando haya recursos): detecta errores de programación que corromperían el estado, **no** valida entradas del jugador (eso es `Command::validate`).

- **API:** `Sim::check_invariants() -> Result<(), Violation>` devuelve la **primera** violación; `Violation` dice el invariante (nombre estable `dominio.regla`, p. ej. `time.monotonic`), la entidad (`event#7`, `command#3`, `system.<id>`, `sim.time`) y el detalle en español. El trait `Invariant` es interno al crate a propósito: el catálogo crece con cada módulo y no debe quedar fijado en la API pública.
- **Cuándo corre solo:** `Sim::advance` lo corre antes y después de cada paso **sólo** en tests y compilaciones con `debug_assertions` y devuelve `AdvanceError::InvariantViolated`. En release **no** corre (costo cero). Cada chequeo es O(estado): es para desarrollo, no para cada cuadro. No corre en `Sim::new` ni en `Sim::submit_command`.
- **Agregar un invariante:** en el módulo dueño del estado, una función `fn check_xxx(sim: &Sim) -> Result<(), Violation>` y una entrada en su constante `INVARIANTS` (`FnInvariant::new("modulo.nombre", check_xxx)`). El nombre es estable y no se renombra. Tiene que ser barato, determinista y sin efectos (sin `HashMap`, sin `unwrap`/`panic!`). Agregá un test `cfg(test)` que corrompa el estado a propósito (hay helpers `*_for_test`) y verifique nombre y entidad exactos. El módulo nuevo se suma a `invariants::all`.
- **Estado que el chequeo necesita recordar** (como el mojón de tiempo de `time.monotonic`) es contabilidad derivada: **no** entra al hash de estado (ver «Qué es "estado"...» arriba); verificá que `state_hash()` no cambie.
- Detalle y política completos en el módulo interno `crates/heliarca-sim/src/invariants.rs`.

## Alcance
La lista de **excluidos del MVP** (doc 21 §1) es una regla: combate, espionaje, mercados grises, varias especies, diplomacia con líderes, regiones en la Tierra, órbitas elípticas, etc. Cualquier idea nueva va al backlog posterior, no al MVP.
