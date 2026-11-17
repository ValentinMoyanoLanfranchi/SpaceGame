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
   - En `heliarca-sim` están prohibidos `HashMap` y `HashSet` (y `RandomState`/`DefaultHasher`): su orden de iteración cambia entre ejecuciones y rompería el determinismo. Usar `BTreeMap`/`BTreeSet`, o `IndexMap`/`IndexSet` (crate `indexmap`, a agregar cuando haga falta).
   - Se hace cumplir con `clippy::disallowed_types` en `crates/heliarca-sim/clippy.toml`; el CI corre `cargo clippy ... -D warnings` y falla si aparecen, también en tests. `heliarca-cli` y `heliarca-godot` no están alcanzados.
   - Clippy no mira dependencias: un tipo de otra librería con hasher aleatorio interno no se detecta, hay que revisarlo en code review.
3. Dirigida por datos: bienes, recetas, techs, diseños, cuerpos y textos en JSON.
   - **Regla: todo número del GDD es un parámetro.** Lo que el diseño pueda querer ajustar (o que el GDD fije como convención: días por semana, mes y año, doc 22 §0) vive en `data/params.json` y se lee vía `Sim::params()`; nunca como literal en el código. Cambiar el JSON cambia el comportamiento sin recompilar.
   - Quedan como constantes de código sólo las **definiciones de unidad** o identidades físicas que ningún diseño ajusta (60 s por minuto, 60 min por hora, 24 h por día). Ante la duda: si el diseño podría querer otro valor, es un parámetro.
   - Para agregar un parámetro: campo en `Params` (con la unidad en el nombre) + valor por defecto del GDD + validación de rango + entrada en `data/params.json` (todos son obligatorios; un campo faltante o desconocido es error) + esquema JSON si el repo ya los genera + tests. Detalle en el módulo `heliarca_sim::params`.
4. El jugador actúa por **comandos**; la UI lee **modelos de vista**, no el estado interno.
5. IDs tipados y arenas; errores con `Result`; `clippy` y `rustfmt` obligatorios.

## Alcance
La lista de **excluidos del MVP** (doc 21 §1) es una regla: combate, espionaje, mercados grises, varias especies, diplomacia con líderes, regiones en la Tierra, órbitas elípticas, etc. Cualquier idea nueva va al backlog posterior, no al MVP.
