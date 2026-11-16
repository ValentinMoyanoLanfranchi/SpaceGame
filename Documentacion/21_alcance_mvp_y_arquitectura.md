# 21 — Alcance del MVP y arquitectura técnica

> v0.2 · Plan para un **desarrollador solo, a tiempo completo (~40 h/semana)**, con **núcleo de simulación en Rust**, **Godot 4 + godot-rust (gdext)** para la interfaz, y Claude escribiendo el código de los módulos a partir de los docs del GDD. Las estimaciones son rangos de referencia (±40%), no compromisos. Los datos concretos del MVP están en 22.

## Resumen de decisiones

| Tema | Resolución |
|---|---|
| Lenguaje del núcleo | **Rust** (optimización de la simulación) |
| Motor / UI | **Godot 4** con extensión **godot-rust (gdext)** |
| Equipo | **Una persona**, tiempo completo |
| Rol de Claude | **Escribe el código del prototipo** a partir de los docs; el desarrollador revisa, prueba y ajusta el *feeling* |
| Alcance del MVP | **Fase 1 jugable**, desde la Tierra hasta **cruzar el primer punto de salto** (datos en 22) |
| Estimación | **Primer ciclo jugable en ~4,5–5,5 meses; MVP completo en ~7–10 meses** (Rust suma ~1 mes por la integración y la curva de aprendizaje) |

---

## 1. Definición del MVP

### Qué es
Una partida completa de la **Fase 1** (02): inicio en la Tierra sin datos del sistema, investigación y construcción desde cero, exploración del sistema, colonias, economía mixta mínima, programas con empresas privadas y cierre al **localizar y cruzar un punto de salto**.

### Alcance incluido

| Área | Contenido del MVP |
|---|---|
| **Mundo** | **Sistema Solar real + elementos ficticios**: 21 cuerpos, 2 asteroides ficticios con un exótico y **un punto de salto oculto** (22). Órbitas circulares en rieles (04). |
| **Conocimiento** | Estados Desconocido → Sospecha → Observado → Relevado → Explorado, con telescopios y relevamiento gravimétrico por zonas (02, 22). |
| **Tiempo** | Incrementos 5 s · 1 h · 1 d · 5 d · 30 d, con pausa e interrupciones por eventos (01). |
| **Investigación** | Proyectos con laboratorios y científicos; **38 tecnologías** de la ruta hasta el cruce (+3 iniciales) cargadas desde datos (13, 22). |
| **Diseño y producción** | 15 diseños de fábrica, edición simple (clonar y cambiar componentes), 17 instalaciones y un taller orbital como astillero (07, 22). |
| **Logística** | Flotas con órdenes manuales; combustible; movimiento entre cuerpos; transporte de carga; costos de lanzamiento (22). |
| **Economía** | **16 bienes** (subconjunto de 18), recetas simplificadas, nodos con stock y precio, mercado terrestre, **3–4 empresas privadas simples**. Impuestos: renta global + regalías de extracción. |
| **Población y sociedad** | **La Tierra como un único nodo, sin regiones ni bloques** (2.500 M de habitantes), colonias dependientes en la Luna y Marte, **apoyo público** y 4 grupos de interés (09, 22). |
| **Programas** | Minería, relevamiento y transporte, en modo propio o contratado (07). |
| **Presiones** | Escasez de recursos, costo del programa espacial, tensión social (02). |
| **Guía** | Objetivos sugeridos opcionales y ayudas contextuales (02, 16). |
| **UI** | Mapa 2D del sistema; paneles de investigación, economía, diseño, flotas, colonias, programas, alertas y crónica (16). |
| **Persistencia** | Guardar y cargar partidas con versión de formato. |

### Alcance **excluido** (después del MVP)
Combate y seguridad (11) · espionaje y golpes (20) · mercados grises y leyes de trabajo (19) · puntos de salto propios · varias especies e inicios (08, 14) · galaxia procedural · diplomacia con líderes y banda de equilibrio de IA (10, 20) · regiones abstractas en la Tierra · órbitas elípticas · mano de obra detallada · tecnologías de Fase 2+ · multijugador.

### MVP+ (opcional, si hay tiempo)
Un **primer contacto** con una IA simple más allá del punto de salto: protocolo de paz, idiomas por fases y relación visible (02, 10). Estimación adicional: **+4–6 semanas**.

### Criterios de aceptación
1. Un jugador nuevo puede ir de la Tierra a cruzar el punto de salto **sin ayuda externa** y sin quedarse atascado.
2. La simulación **sin interfaz** corre **40 años de juego en menos de 30 s** en el escenario del MVP (objetivo de Rust).
3. Se puede **guardar y cargar** en cualquier momento sin corrupción.
4. Existen **al menos dos estilos viables** (más estatal y más con privados) para llegar al punto de salto.
5. Los **invariantes** de la simulación se cumplen (no se crean ni destruyen recursos sin receta).
6. Todas las decisiones importantes tienen **explicación visible** ("¿por qué?", informes de cuellos de botella; 16).

---

## 2. Arquitectura técnica (Rust + Godot)

### Principios
1. **Simulación separada del render.** El núcleo es un **crate de Rust sin dependencias de Godot**: se ejecuta y prueba en consola.
2. **Determinista.** Dada una semilla y una lista de comandos, siempre produce lo mismo: RNG con semilla (p. ej. PCG), orden estable de iteración (`BTreeMap`/`IndexMap`, no `HashMap` con orden aleatorio), `f64` y sin tiempo real en la lógica.
3. **Dirigida por datos.** Bienes, recetas, tecnologías, diseños, cuerpos, eventos y textos viven en **archivos JSON** (22), no en el código.
4. **El jugador actúa por comandos.** La UI emite **comandos** (investigar X, construir Y, enviar flota Z) que la simulación valida y ejecuta: permite pruebas, repeticiones, IA y *autoplayer*.
5. **La UI lee instantáneas / modelos de vista**, no el estado interno.
6. **Todo número del GDD es un parámetro** en archivos de datos.

### Estructura del repositorio (workspace de Cargo)

```
heliarca/
  docs/gdd/                  # copia del GDD (referencia)
  data/                      # JSON del MVP (22)
  Cargo.toml                 # workspace
  crates/
    heliarca-sim/            # núcleo de simulación (sin Godot)
    heliarca-cli/            # ejecutor sin interfaz (balance, pruebas, autoplayer)
    heliarca-godot/          # extensión gdext (cdylib): puente con Godot
  tests/                     # escenarios e invariantes
  godot/                     # proyecto Godot: escenas, UI, .gdextension
```

### Crates y herramientas sugeridas

| Necesidad | Opción |
|---|---|
| Serialización de datos y guardado | `serde` + `serde_json`; compresión `zstd` |
| Validación de datos | `schemars` (esquemas JSON) + validación al cargar |
| RNG determinista | `rand` + `rand_pcg` |
| Identificadores y arenas | IDs tipados (`u32` nuevos tipos) y `slotmap` o `Vec` indexados |
| Orden estable | `indexmap`, `BTreeMap` |
| Paralelismo (opcional) | `rayon` en pasos independientes (economía por nodo), con **reducción determinista** |
| Pruebas | `cargo test`, `proptest` (invariantes), `criterion` (benchmarks) |
| Errores | `thiserror` / `anyhow` en la CLI |
| Puente con Godot | `godot` (gdext) |

Se evita un ECS completo al inicio: **estructuras simples con IDs** son más fáciles de razonar y de generar con Claude; si hace falta, se migra a un ECS (`hecs` / `bevy_ecs` sin renderer).

### Módulos del núcleo (`heliarca-sim`)

| Módulo | Contenido | Docs |
|---|---|---|
| **core** | Reloj, planificador multi-frecuencia, RNG, bus de eventos, comandos, serialización | 01 |
| **world** | Cuerpos, órbitas en rieles, estados de conocimiento, relevamiento por zonas | 04, 02, 22 |
| **research** | Laboratorios, científicos, proyectos, tecnologías | 13 |
| **economy** | Bienes, recetas, instalaciones, nodos, mercados, precios, mercado terrestre | 03, 18, 22 |
| **logistics** | Flotas, órdenes, combustible, rutas, transporte, lanzamientos | 04 |
| **design** | Componentes, diseños, diseños de fábrica, astillero | 07 |
| **society** | Pops agregados, grupos de interés, apoyo público, colonias | 05, 09 |
| **companies** | Empresas privadas, contratos, retorno esperado | 03, 19 |
| **programs** | Programas (minería, relevamiento, transporte) y asignación | 07 |
| **scenario** | Inicios y carga de datos | 02, 22 |
| *(luego)* **factions / combat / espionage** | IA, líderes, combate, seguridad | 10, 11, 20 |

### Planificador multi-frecuencia (01)
La simulación avanza por **incrementos** (5 s, 1 h, 1 d, 5 d, 30 d). Cada subsistema corre a su cadencia:

| Subsistema | Cadencia inicial |
|---|---|
| Movimiento de flotas, eventos de proximidad | Cada incremento |
| Investigación, producción, consumo | Diaria |
| Precios y mercado | Semanal |
| Empresas privadas, sociedad | Mensual |

Los eventos relevantes (descubrimiento, llegada, crisis) **interrumpen** el avance y se encolan como alertas.

### Puente con Godot (gdext)
- Una clase expuesta (p. ej. `SimHandle`) que **posee la simulación** y ofrece pocas llamadas gruesas: `load_scenario`, `submit_command(json)`, `advance(increment)`, `poll_events()`, y **consultas de modelos de vista** (por pantalla) devueltas como `Dictionary` / `PackedArray`, no objeto por objeto.
- **Poco cruce por frame:** la UI pide datos cuando cambian (por evento o cada cierto tiempo), no cada frame.
- **Capa fina:** toda la lógica vive en `heliarca-sim`; `heliarca-godot` sólo traduce tipos.
- **Versiones:** se fija la versión de Godot y de gdext (el API de gdext sigue la versión de Godot). Se prueba **recarga en caliente** del editor.
- La UI se construye con escenas de Godot; la lógica de presentación (formateo, listas) en GDScript o en Rust.

### Datos y localización
- IDs **estables en texto** (`tech.pro_0_1`, `good.hierro`) con tabla de correspondencia con los números de 18.
- Textos por **claves de localización** desde el inicio (español primero).
- Validación de esquemas al cargar; errores claros si un dato es inválido.

### Guardado
- **JSON comprimido, versionado**, con migraciones. Más adelante, formato binario si hace falta.
- La partida guarda estado + semilla (+ historial de comandos relevantes).

### Pruebas y herramientas de balance
- **Unitarias** por módulo; **invariantes** con `proptest` (conservación de recursos, saldos, referencias).
- **Escenario**: "desde el inicio, un *autoplayer* simple llega a X en N años".
- **Autoplayer / bots**: estrategias (estatal, privada, mixta) para detectar bloqueos y desbalances.
- **Telemetría**: la CLI exporta CSV (precios, PI/mes, población, apoyo público) para graficar.
- **Benchmarks** (`criterion`) para vigilar el rendimiento.

### Rendimiento
- Presupuesto del MVP: **1 año de juego en < 0,75 s** en modo sin interfaz (≈ 40 años en < 30 s); avance de 30 días fluido en la UI.
- LOD de simulación (01) sólo cuando haga falta; en el MVP el número de entidades es chico, así que el **cuello de botella probable no es el cómputo sino el diseño y la UI**. Rust protege el margen para escalar (Fase 2 y 3).

---

## 3. Plan por hitos (una persona, tiempo completo)

| Hito | Contenido | Semanas |
|---|---|---|
| **M0 — Cimientos** | Workspace de Cargo, crate de simulación, CLI, pruebas, carga de datos JSON, reloj y planificador, comandos, guardado básico, **extensión gdext mínima funcionando en Godot** | 2 |
| **M1 — Mundo** | Cuerpos y órbitas en rieles, mapa 2D, controles de tiempo, estados de conocimiento y telescopios | 3–4 |
| **M2 — Investigación** | Laboratorios, científicos, proyectos, 38 tecnologías desde datos, panel de investigación | 4–5 |
| **M3 — Economía mínima** | 16 bienes, recetas, instalaciones, nodos, stock y precios, mercado terrestre, logística básica, telemetría | 5–6 |
| **M4 — Diseño y flotas** | Diseños de fábrica, edición simple, lanzamientos, taller orbital, flotas, combustible, órdenes manuales | 4–5 |
| *(hito: **primer ciclo jugable**)* | Investigar → lanzar sonda → relevar un cuerpo → ver datos | *≈ 18–22 acumuladas* |
| **M5 — Sociedad** | Tierra como nodo, colonias Luna/Marte, apoyo público, grupos de interés, presiones | 3–4 |
| **M6 — Privados y programas** | Empresas privadas, contratos, programas de minería, relevamiento y transporte | 4–5 |
| **M7 — Cierre de la Fase 1** | Relevamiento gravimétrico por zonas, punto de salto, EXP-2, cruce, objetivos sugeridos, guardado completo | 3–4 |
| **M8 — Pulido y balance** | Calibración, *autoplayer*, errores, rendimiento, ayudas contextuales | 4–6 |
| **Total** | | **~33–42 semanas (≈ 7,5–10 meses)** |

> La ayuda de Claude acelera la escritura de código; la revisión, la UI y el balance siguen siendo trabajo humano. Rust añade **curva de aprendizaje y tiempo de compilación**, pero reduce errores en tiempo de ejecución y da margen de rendimiento. La mayor incertidumbre está en la **interfaz** y el **balance**, no en la simulación.

### Orden de trabajo recomendado
1. Hacer M0–M3 **con la CLI** y pruebas, y sólo una interfaz mínima de verificación, para validar economía e investigación antes de invertir en pantallas.
2. Agregar la UI por hito, no al final.
3. Mantener un **autoplayer** desde M2 para detectar bloqueos temprano.

---

## 4. Flujo de trabajo con Claude

1. **Spec a código por módulo:** cada módulo se pide con referencia a los docs y datos ("implementá `research` según 13 v0.3 y las 38 tecnologías de 22").
2. **Siempre con pruebas:** cada módulo trae pruebas; sin ellas no se acepta.
3. **Cambios pequeños y revisables:** un módulo o funcionalidad por iteración; se compila y prueba antes de seguir.
4. **El GDD es la fuente de verdad:** cuando una decisión de implementación cambia el diseño, se actualiza el doc.
5. **División de tareas sugerida:**
   - Claude: crates de simulación, datos, pruebas, extensión gdext y lógica de modelos de vista.
   - Vos: instalar y compilar el proyecto, construir/ajustar escenas y estilos en el editor de Godot, probar el *feeling*, decidir cambios de diseño.
6. **Estilo Rust para facilitar la generación:** IDs tipados y arenas (evitar grafos de referencias), errores con `Result`, módulos pequeños, `clippy` y `rustfmt` obligatorios.
7. **Verificación:** el crate de simulación se compila y prueba con `cargo test`; la parte de Godot se prueba en el editor.

---

## 5. Primeros pasos concretos

1. Instalar **Rust** (rustup), **Godot 4** (estable) y las herramientas de compilación del sistema.
2. Crear el workspace con la estructura de arriba y fijar la versión de gdext que corresponda a la versión de Godot.
3. Hacer M0 con Claude: crate de simulación, CLI, pruebas, carga de datos JSON, reloj, planificador, comandos, y una extensión gdext que muestre "hola" desde el núcleo.
4. Empezar por los datos de 22: cuerpos, bienes y las 38 tecnologías en JSON.

---

## Riesgos específicos del desarrollo

| Riesgo | Mitigación |
|---|---|
| **Crecer el alcance** (agregar combate, diplomacia, etc. antes del MVP) | La lista de excluidos es una regla; todo lo nuevo va al backlog posterior |
| **Interfaz más cara de lo previsto** | UI mínima y funcional; reutilizar componentes; priorizar tablas y tooltips sobre arte |
| **Balance difícil** | Parámetros en datos, CLI, *autoplayer* y telemetría desde temprano |
| **Código generado inconsistente** | Pruebas obligatorias, módulos chicos, `clippy`, revisión humana, el GDD como referencia |
| **gdext todavía en maduración** | Capa de puente fina; núcleo y CLI independientes; fijar versiones; si falla, la CLI y una UI alternativa (web/Tauri) siguen siendo posibles |
| **Curva de Rust y borrow checker** en estructuras de simulación | IDs y arenas en lugar de referencias; Claude escribe el patrón base; revisar pronto |
| **Tiempo de compilación** | Workspace separado, `cargo check`, linker rápido, perfil de desarrollo optimizado |
| **Determinismo roto** | RNG con semilla, orden estable, pruebas de repetición |
| **Quemarse trabajando solo** | Hitos cortos y verificables; primer ciclo jugable temprano |

## Conexiones
01 (planificador, LOD) · 02 (Fase 1) · 07 (diseños, programas) · 13 (investigación) · 16 (UI) · 17 (decisiones y ruta crítica) · 18 (bienes) · 22 (datos del MVP).

## Preguntas abiertas
- Confirmar con un pequeño prototipo la integración gdext + Godot (versión y recarga en caliente) antes de comprometer toda la UI.
- Formato de datos: ¿JSON puro o con herramientas de edición (hoja de cálculo exportable)?
- ¿Qué parte de la UI va en escenas del editor y cuál de forma programática?
- Arte: ¿iconos y estilo visual mínimo (placeholders) hasta el MVP?
- ¿Hay `cargo` disponible (y acceso a los crates) en el entorno de Claude para compilar y probar en la conversación? Si no, correr las pruebas localmente.
