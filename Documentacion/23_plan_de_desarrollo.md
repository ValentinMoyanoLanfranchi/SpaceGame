# 23 — Plan de desarrollo del MVP

> v0.1 · Traduce el alcance y el plan por hitos de **21** y los datos de **22** a un backlog ejecutable en GitHub. Este documento define **cómo se descompone y en qué orden se construye**; el detalle de cada tarea (requisitos y criterios de aceptación) vive en los issues. Estimaciones: rangos de referencia (±40%), no compromisos.

## 1. Jerarquía

```
Hito (Milestone M0…M8)  ──  una entrega verificable
 └─ Épica    (type:epic)   ──  una funcionalidad grande y cerrable
     └─ Historia (type:story) ──  valor entregable ("Como… quiero… para…")
         └─ Tarea (type:task)  ──  ≤ ~1 día, con requisitos y criterios de aceptación
```

- Cada tarea es **un PR**. Si no cabe en ~1 día, se divide (`size:L` es el techo).
- Los códigos de los títulos siguen el árbol: `M2.3.1.4` = hito 2 · épica 3 · historia 1 · tarea 4.
- **Gradualidad:** las dependencias van siempre hacia atrás (ninguna tarea depende de una con número mayor). Se puede trabajar en orden numérico sin bloquearse.
- Vistas útiles en GitHub: filtrar por *milestone* + `type:task`; o por `area:*` para ver una capa.

## 2. Orden de construcción (principios)

1. **CLI primero (M0–M3).** Economía e investigación se validan sin pantallas; sólo hay una UI mínima de verificación (doc 21 §3).
2. **UI por hito, no al final.** Cada hito trae su panel.
3. **Autoplayer desde M2** para detectar bloqueos temprano.
4. **Invariantes y determinismo desde M0**: se prueban en cada hito, no se dejan para el pulido.
5. **Los números son datos.** Primero la tabla de 22 en JSON validado, luego la lógica que la consume.
6. **Lo excluido no se hace** (doc 21 §1): va al backlog posterior.

## 3. Hitos y condiciones de salida

| Hito | Contenido | Sem. | Condición de salida (gate) |
|---|---|---|---|
| **M0 Cimientos** | Workspace, núcleo, CLI, datos, guardado básico, gdext mínimo | 2 | Escenario vacío corre 40 años en la CLI; save/load idéntico; Godot muestra un "hola" del núcleo |
| **M1 Mundo** | 21 cuerpos, órbitas en rieles, conocimiento, telescopios, mapa 2D, tiempo, alertas | 3–4 | Con el equipo inicial se sospechan Venus, Marte, Júpiter (6 m), Mercurio y Saturno (8 m); se ve en el mapa |
| **M2 Investigación** | Labs, científicos, proyectos, 38+3 techs, panel, autoplayer v0 | 4–5 | El autoplayer completa la ruta de techs; ≈120 PI/mes iniciales |
| **M3 Economía mínima** | 16 bienes, recetas, instalaciones, nodos, precios, mercado terrestre, logística básica | 5–6 | Cadena hierro→acero→piezas→repuestos funciona; invariantes de conservación en verde; año de juego < 0,75 s |
| **M4 Diseño y flotas** | 15 diseños, edición simple, taller orbital, flotas, combustible, órdenes | 4–5 | **Primer ciclo jugable:** investigar → lanzar sonda → relevar un cuerpo → ver datos |
| **M5 Sociedad** | Tierra única, apoyo público, grupos, colonias Luna/Marte | 3–4 | Se funda una colonia y se sostiene; el apoyo público limita el presupuesto espacial |
| **M6 Privados y programas** | 4 empresas, contratos, impuestos, programas | 4–5 | Un programa de minería contratado a privados abastece una colonia; hay informe de cuellos de botella |
| **M7 Cierre de la Fase 1** | Gravimetría por zonas, punto de salto, EXP-2, cruce, objetivos, guardado completo | 3–4 | Se cruza el punto de salto y aparece "Fase 1 completada" |
| **M8 Pulido y balance** | Calibración, rendimiento, robustez, ayudas, playtest | 4–6 | Se cumplen los **6 criterios de aceptación del MVP** (doc 21 §1) |
| **MVP+** *(opcional)* | Primer contacto con una IA simple | +4–6 | — |

### Criterios de aceptación del MVP (doc 21 §1) → dónde se verifican
1. Un jugador nuevo llega al cruce sin ayuda externa ni atascos → M8 (playtest).
2. 40 años de juego en < 30 s sin interfaz → M3 (benchmark) y M8 (perfilado).
3. Guardar/cargar sin corrupción en cualquier momento → M0, M7 y M8 (fuzz).
4. Al menos dos estilos viables (estatal y con privados) → M8 (autoplayers).
5. Invariantes de conservación → M0 (marco), M3 (bienes y créditos) y M8 (barrido).
6. Toda decisión importante con explicación visible ("¿por qué?") → transversal; revisión en M8.

## 4. Ruta crítica

`M0 núcleo y datos → M1 cuerpos/conocimiento → M2 investigación → M3 economía y lanzamiento → M4 diseños y flotas (primer ciclo) → M5 colonias → M6 programas → M7 gravimetría y EXP-2 → cruce`.

Lo que **puede** ir en paralelo (con una sola persona, intercalar para descansar de un área): UI de un hito mientras se prueba su lógica; datos JSON de hitos siguientes; benchmarks y autoplayer.

## 5. Decisiones pendientes que afectan al MVP
Están abiertas como issues `type:decision` / `type:spike` con su hito:

| Tema | Doc | Se decide en |
|---|---|---|
| Versión de Godot + gdext y recarga en caliente | 21, 17 #38, 24 | M0 (versión decidida en 24; recarga en caliente en M0.7.1.4) |
| ¿Hay `cargo` disponible en el entorno de Claude? | 17 #40 | M0 |
| Formato de datos: ¿JSON puro o con hoja de cálculo exportable? | 21 | M0 |
| Escenas del editor vs. UI programática; estilo visual mínimo | 21, 17 #39 | M1 |
| Lunas de Júpiter y Titán: ¿sólo observables? ¿SEN-1.3 dentro del MVP? | 22, 17 #36 | M1 |
| ¿Se agrega ADM-0.2 o se supone flujo fijo de graduados? | 22, 17 #35 | M2 |
| Costo de lanzamiento vs. presupuesto (~20 lanzamientos/año) | 22, 17 #13 | M3 |
| Probabilidades base de fallo de lanzamiento y de misión | 22, 17 #37 | M4 |
| Reparto propio/contratado en programas mixtos | 07, 17 #12 | M6 |

## 6. Convenciones transversales
- **IDs de texto estables** (`body.luna`, `good.hierro`, `tech.pro_0_1`, `design.lv_1`), con tabla de correspondencia con los números de 18.
- **Textos por clave de localización** desde el inicio (español primero).
- **Cadencias** (21 §2): flotas y proximidad en cada incremento; investigación, producción y consumo diarios; precios semanales; empresas y sociedad mensuales.
- **Estructura del repo** (21 §2): el workspace de Cargo vive en la raíz (`crates/heliarca-sim`, `crates/heliarca-cli`, `crates/heliarca-godot`), con `data/`, `tests/` y `godot/`. `Documentacion/` se mantiene donde está.
- **Calibración:** todo número inicial es provisorio; se ajusta en M8 con la CLI, el autoplayer y la telemetría.

## 7. Índice de issues

Generado al cargar el backlog. Cada hito enlaza a su *milestone* (lista completa con las tareas); aquí se listan épicas e historias con el número de tareas de cada una.

### [M0 — Cimientos](https://github.com/ValentinMoyanoLanfranchi/SpaceGame/milestone/1)

- **#3 `M0.1` Repositorio, toolchain y CI** — 2 historias · 8 tareas
  - #4 `M0.1.1` Workspace reproducible (4 tareas)
  - #9 `M0.1.2` CI de calidad en cada PR (4 tareas)
- **#14 `M0.2` Núcleo de simulación: tiempo, determinismo y comandos** — 4 historias · 10 tareas
  - #15 `M0.2.1` Reloj y planificador multi-frecuencia (2 tareas)
  - #18 `M0.2.2` Determinismo y orden estable (2 tareas)
  - #21 `M0.2.3` IDs, eventos y comandos (4 tareas)
  - #26 `M0.2.4` Avance con interrupciones y repetibilidad (2 tareas)
- **#29 `M0.3` Datos y localización** — 2 historias · 5 tareas
  - #30 `M0.3.1` Carga y validación de datos (3 tareas)
  - #34 `M0.3.2` Localización y parámetros (2 tareas)
- **#37 `M0.4` CLI sin interfaz** — 1 historias · 4 tareas
  - #38 `M0.4.1` Ejecutar y validar sin interfaz (4 tareas)
- **#43 `M0.5` Pruebas, invariantes y benchmarks** — 1 historias · 3 tareas
  - #44 `M0.5.1` Infraestructura de calidad desde el día uno (3 tareas)
- **#48 `M0.6` Guardado y carga básicos** — 1 historias · 5 tareas
  - #49 `M0.6.1` Guardar y continuar (5 tareas)
- **#55 `M0.7` Puente Godot (gdext) mínimo** — 2 historias · 7 tareas
  - #56 `M0.7.1` Núcleo dentro de Godot (5 tareas)
  - #62 `M0.7.2` Decisiones de arranque cerradas (2 tareas)

### [M1 — Mundo](https://github.com/ValentinMoyanoLanfranchi/SpaceGame/milestone/2)

- **#65 `M1.1` Datos del Sistema Solar del MVP** — 1 historias · 8 tareas
  - #66 `M1.1.1` Cuerpos, yacimientos y punto de salto como datos (8 tareas)
- **#75 `M1.2` Órbitas en rieles** — 1 historias · 6 tareas
  - #76 `M1.2.1` Posición de los cuerpos en el tiempo (6 tareas)
- **#83 `M1.3` Conocimiento del sistema y telescopios** — 2 historias · 9 tareas
  - #84 `M1.3.1` Modelo de conocimiento y vistas con incertidumbre (4 tareas)
  - #89 `M1.3.2` Telescopios y revelado por observación (5 tareas)
- **#95 `M1.4` Alertas, registro y explicación causal** — 1 historias · 4 tareas
  - #96 `M1.4.1` Alertas, auto-pausa y crónica mínima (4 tareas)
- **#101 `M1.5` Mapa 2D del sistema y controles de tiempo** — 3 historias · 12 tareas
  - #102 `M1.5.1` Puente de lectura y decisiones de UI (4 tareas)
  - #107 `M1.5.2` Mapa del sistema (4 tareas)
  - #112 `M1.5.3` Tiempo, alertas y conocimiento en pantalla (4 tareas)

### [M2 — Investigación](https://github.com/ValentinMoyanoLanfranchi/SpaceGame/milestone/3)

- **#117 `M2.1` Tecnologías desde datos** — 2 historias · 8 tareas
  - #118 `M2.1.1` Catálogo de tecnologías como datos (5 tareas)
  - #124 `M2.1.2` Estado de tecnologías y efectos (3 tareas)
- **#128 `M2.2` Laboratorios, científicos y proyectos** — 2 historias · 10 tareas
  - #129 `M2.2.1` Laboratorios y científicos (3 tareas)
  - #133 `M2.2.2` Proyectos de investigación (7 tareas)
- **#141 `M2.3` Panel de investigación** — 1 historias · 6 tareas
  - #142 `M2.3.1` Gestionar la investigación desde la interfaz (6 tareas)
- **#149 `M2.4` CLI: telemetría y autoplayer v0** — 1 historias · 3 tareas
  - #150 `M2.4.1` Telemetría y autoplayer de investigación (3 tareas)

### [M3 — Economía mínima](https://github.com/ValentinMoyanoLanfranchi/SpaceGame/milestone/4)

- **#154 `M3.1` Catálogo económico como datos** — 1 historias · 6 tareas
  - #155 `M3.1.1` Bienes, recetas e instalaciones desde JSON (6 tareas)
- **#162 `M3.2` Nodos, inventario y tesorería** — 2 historias · 6 tareas
  - #163 `M3.2.1` Nodos y stock con conservación (3 tareas)
  - #167 `M3.2.2` Instalaciones y construcción (3 tareas)
- **#171 `M3.3` Producción, energía y minería** — 1 historias · 6 tareas
  - #172 `M3.3.1` Producción con insumos y energía (6 tareas)
- **#179 `M3.4` Mercado y precios** — 1 historias · 5 tareas
  - #180 `M3.4.1` Precios por nodo según stock (5 tareas)
- **#186 `M3.5` Mercado terrestre y lanzamiento** — 2 historias · 7 tareas
  - #187 `M3.5.1` Comprar y vender en la Tierra (3 tareas)
  - #191 `M3.5.2` Subir carga a órbita (4 tareas)
- **#196 `M3.6` Telemetría, invariantes y rendimiento económico** — 1 historias · 5 tareas
  - #197 `M3.6.1` Economía verificable (5 tareas)
- **#203 `M3.7` Panel de economía mínimo** — 1 historias · 5 tareas
  - #204 `M3.7.1` Ver y operar la economía (5 tareas)

### [M4 — Diseño y flotas](https://github.com/ValentinMoyanoLanfranchi/SpaceGame/milestone/5)

- **#210 `M4.1` Diseños de fábrica** — 1 historias · 6 tareas
  - #211 `M4.1.1` Catálogo de diseños de fábrica (6 tareas)
- **#218 `M4.2` Edición simple de diseños** — 1 historias · 4 tareas
  - #219 `M4.2.1` Clonar y editar diseños (4 tareas)
- **#224 `M4.3` Flotas, movimiento y combustible** — 1 historias · 7 tareas
  - #225 `M4.3.1` Naves y movimiento (7 tareas)
- **#233 `M4.4` Construcción y taller orbital** — 1 historias · 3 tareas
  - #234 `M4.4.1` Construir y poner en órbita (3 tareas)
- **#238 `M4.5` Misiones, sondas y satélites** — 1 historias · 5 tareas
  - #239 `M4.5.1` Sondas, satélites y fallos de misión (5 tareas)
- **#245 `M4.6` Interfaz de diseños y flotas** — 1 historias · 5 tareas
  - #246 `M4.6.1` Diseñar, construir y mover desde la UI (5 tareas)
- **#252 `M4.7` Hito: primer ciclo jugable** — 1 historias · 3 tareas
  - #253 `M4.7.1` Primer ciclo jugable verificado (3 tareas)

### [M5 — Sociedad](https://github.com/ValentinMoyanoLanfranchi/SpaceGame/milestone/6)

- **#257 `M5.1` La Tierra como nodo único** — 1 historias · 3 tareas
  - #258 `M5.1.1` Economía y presupuesto de la Tierra (3 tareas)
- **#262 `M5.2` Grupos de interés y apoyo público** — 3 historias · 9 tareas
  - #263 `M5.2.1` Apoyo público y grupos de interés (6 tareas)
  - #270 `M5.2.2` Eventos ambientales simples (2 tareas)
  - #273 `M5.2.3` Presupuesto ligado al apoyo público (1 tareas)
- **#275 `M5.3` Colonias en la Luna y Marte** — 1 historias · 7 tareas
  - #276 `M5.3.1` Fundar y sostener una colonia (7 tareas)
- **#284 `M5.4` Notables y líder del Estado** — 1 historias · 2 tareas
  - #285 `M5.4.1` Líder del Estado (2 tareas)
- **#288 `M5.5` Interfaz de sociedad** — 1 historias · 5 tareas
  - #289 `M5.5.1` Gestionar sociedad y colonias (5 tareas)

### [M6 — Privados y programas](https://github.com/ValentinMoyanoLanfranchi/SpaceGame/milestone/7)

- **#295 `M6.1` Empresas privadas** — 2 historias · 7 tareas
  - #296 `M6.1.1` Reglas generales de las empresas privadas (4 tareas)
  - #301 `M6.1.2` Comportamiento de las empresas del MVP (3 tareas)
- **#305 `M6.2` Impuestos y licencias** — 2 historias · 3 tareas
  - #306 `M6.2.1` Recaudar y regular (2 tareas)
  - #309 `M6.2.2` Minería privada con licencia (1 tareas)
- **#311 `M6.3` Contratos y compras a privados** — 2 historias · 5 tareas
  - #312 `M6.3.1` Contratos de servicios (3 tareas)
  - #316 `M6.3.2` Comprar naves y tecnología a privados (2 tareas)
- **#319 `M6.4` Programas de órdenes masivas** — 2 historias · 9 tareas
  - #320 `M6.4.1` Núcleo de programas (3 tareas)
  - #324 `M6.4.2` Tipos de programa del MVP (6 tareas)
- **#331 `M6.5` Interfaz de empresas y programas** — 1 historias · 4 tareas
  - #332 `M6.5.1` Gestionar privados y programas (4 tareas)

### [M7 — Cierre de la Fase 1](https://github.com/ValentinMoyanoLanfranchi/SpaceGame/milestone/8)

- **#337 `M7.1` Relevamiento gravimétrico y punto de salto** — 1 historias · 5 tareas
  - #338 `M7.1.1` Encontrar y estudiar el punto de salto (5 tareas)
- **#344 `M7.2` Cristales de presión** — 1 historias · 3 tareas
  - #345 `M7.2.1` Obtener cristales de presión (3 tareas)
- **#349 `M7.3` Cruce del punto de salto y cierre de la Fase 1** — 1 historias · 4 tareas
  - #350 `M7.3.1` Cruzar el punto de salto (4 tareas)
- **#355 `M7.4` Objetivos sugeridos y crónica de hitos** — 1 historias · 3 tareas
  - #356 `M7.4.1` Objetivos opcionales y crónica (3 tareas)
- **#360 `M7.5` Guardado completo y menú de partida** — 1 historias · 4 tareas
  - #361 `M7.5.1` Guardar y cargar todo (4 tareas)

### [M8 — Pulido y balance](https://github.com/ValentinMoyanoLanfranchi/SpaceGame/milestone/9)

- **#366 `M8.1` Autoplayers y detección de bloqueos** — 1 historias · 5 tareas
  - #367 `M8.1.1` Autoplayers por estilo (5 tareas)
- **#373 `M8.2` Calibración y balance** — 1 historias · 6 tareas
  - #374 `M8.2.1` Ajustar los números con datos (6 tareas)
- **#381 `M8.3` Rendimiento** — 1 historias · 3 tareas
  - #382 `M8.3.1` Rápido en la CLI y en la UI (3 tareas)
- **#386 `M8.4` Robustez y calidad** — 1 historias · 5 tareas
  - #387 `M8.4.1` Sin corrupción, sin sorpresas (5 tareas)
- **#393 `M8.5` Ayudas, accesibilidad y localización** — 1 historias · 4 tareas
  - #394 `M8.5.1` Aprender y configurar (4 tareas)
- **#399 `M8.6` Playtest, entrega y documentación** — 1 historias · 5 tareas
  - #400 `M8.6.1` MVP validado y entregable (5 tareas)

### [MVP+ (opcional)](https://github.com/ValentinMoyanoLanfranchi/SpaceGame/milestone/10)

- **#406 `M9.1` Primer contacto con una IA simple (MVP+)** — 0 historias · 0 tareas _(sin dividir; opcional)_

**Totales:** 52 épicas · 70 historias · 282 tareas.
