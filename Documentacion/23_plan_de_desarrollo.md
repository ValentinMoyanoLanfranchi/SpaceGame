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
| Versión de Godot + gdext y recarga en caliente | 21, 17 #38 | M0 |
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
*(Se completa tras cargar el backlog.)*
