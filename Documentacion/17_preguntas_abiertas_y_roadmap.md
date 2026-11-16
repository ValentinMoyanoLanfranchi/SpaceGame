# 17 — Decisiones, riesgos y hoja de ruta

> Última actualización: 2026-11-16. Modelo principal: **Aurora 4X en 2D**, con más mecánicas, recursos, economía y escala. **Todos los docs 00–16 fueron actualizados a v0.3 y están alineados** con el inicio por defecto (Tierra pre-espacial, único nodo), Rust + Godot, el MVP y los docs 18–22. Complementos: **18** (bienes, cadenas y parámetros económicos), **19** (impuestos por categoría, incentivos y mercados grises), **20** (líderes, regímenes y rasgos), **21** (alcance del MVP y arquitectura técnica, v0.2 con Rust) y **22** (datos iniciales del MVP).

## Decisión de base

**El modelo principal es Aurora 4X en 2D.** Se mantiene lo que nos distingue (mundo vivo, logística real, economía profunda, historia emergente), pero el arranque, el rol del jugador, la economía, la diplomacia y el combate se rediseñaron sobre ese modelo. Kenshi/RimWorld/Dwarf Fortress quedan como inspiración secundaria.

---

## Decisiones tomadas

| # | Decisión | Resolución |
|---|---|---|
| 1 | Método FTL | **Puntos de salto.** Algunos naturales son ocultos (13, 14). |
| 2 | Premisa | **Diáspora y Colapso adaptada** (00). Con el inicio por defecto (Tierra), **la premisa describe el mundo que el jugador encuentra más allá de su sistema**; el inicio "Civilización caída" la conserva en su forma original (00, 14). |
| 3 | Población / individuos | **Pops agregados + notables** (05, 06). |
| 4 | Servidumbre | **Sistema opcional con consecuencias** (09, 19). |
| 5 | Rol del jugador | **Estado/imperio**, sin avatar (00, 06). |
| 6 | Tiempo | **Incrementos puros** estilo Aurora, con pausa, interrupciones y ralentización automática ante contactos (01). |
| 7 | Estilo visual | **2D** (16). |
| 8 | Combate | **Espacial táctico 2D + terrestre abstracto**, con doctrinas, ventanas de decisión, resolución rápida y alertas anticipadas (11). |
| 9 | Realismo orbital | **Estilo Aurora**: órbitas en rieles, combustible por distancia (04). |
| 10 | Avatar | **Ninguno.** |
| 11 | Especies iniciales | **3–4 bien hechas**; propuesta: Humanos, Kethari, Ossari, Drel (08). Humanos = default. |
| 12 | Saltar fases | **Superada**: ver decisión 43. |
| 13 | Planetas | **Regiones abstractas por planeta**, sin mapa de superficie (05, 12, 14). **En el MVP la Tierra es un único nodo** (decisión 60). |
| 14 | Economía | **Mercado complejo y mixto**: empresas estatales + privadas, compras estatales, impuestos, importación, comercio, pactos con otras razas, razas totalmente estatales (03). |
| 15 | Estilos de juego | **Estatal, capitalista, mixto, imperialista, cliente/protectorado, autárquico** (00, 03). |
| 16 | Arranque | Ver decisión 43 (reemplaza "presencia mínima en el sistema"). |
| 17 | Fases | **3: Sistema local → Interestelar → Sector** (02). |
| 18 | Planeta capital | **Unificado bajo el jugador** al inicio (02, 09). |
| 19 | Líderes y diplomacia | **Modelo EU4 + CK3**: líder por facción; actitud = especie/cultura + régimen + líder (06, 09, 10, 20). |
| 20 | Golpes y asesinatos | **Operaciones de espionaje con probabilidad** (10, 20). |
| 21 | Información entre sistemas | **Latencia mitigable con relés** y sensores (01, 13). |
| 22 | Cantidad de bienes | **~73 en el núcleo**, extensible por mods (03, 18). |
| 23 | Puntos de salto propios | **Inversión media, escalable** (04, 13). |
| 24 | Minerales | **Reales con pocos ficticios** (18). |
| 25 | Empresas estatales | **Modelo híbrido** (03, 18). |
| 26 | Entrada de privados | **Por rentabilidad**, con rubros reservados, licencias y regalías (03, 18). |
| 27 | Precios | **Por nodo según stock objetivo** (03, 18). |
| 28 | Bienes de cambio | **Sector por sector**; índice sectorial + ajuste por nodo (18). |
| 29 | Impuestos por categoría | **Global + grupo + sector + bien, con herencia** (19). |
| 30 | Incentivos | Bajar impuestos mueve el **retorno esperado** de los privados; subsidios, créditos, precio de sostén, contratos, zonas de fomento, inversión estatal (19). |
| 31 | Sustancias y zonas grises | **Sistema completo** con sustancias ficticias (19). |
| 32 | Leyes de trabajo | **Parámetros con trade-offs**, abstractos; "contenido sensible" configurable (19, 14). |
| 33 | Detección | **Sensibilidad vs. firma**; alcance ≈ 0,5 × √(S × F) Mkm (11). |
| 34 | Resolución rápida | **Desde 4:1**, configurable (11). |
| 35 | Cazas y drones | **Portados** (11, 07). |
| 36 | Ventanas de decisión | **Umbral de amenaza configurable** (11). |
| 37 | Seguridad (0–100) | `30 Cobertura + 25 Patrullas + 20 Respuesta + 15 Defensas + 10 Minas` (11). |
| 38 | Rasgos de líder | **Lista fija con opuestos**, 15 pares, vector de 4 ejes (20). |
| 39 | Regímenes | **7 arquetipos** (20). |
| 40 | Sucesión | **Según el régimen**, candidatos 3–5 (20). |
| 41 | Visibilidad de líderes ajenos | **Niveles N0–N3** por inteligencia (20). |
| 42 | Fórmulas de espionaje | `p_éxito` y `p_descubrimiento` iniciales (20). |
| 43 | **Inicio por defecto** | **Tierra, 2–3 mil millones de habitantes, sin tecnología espacial y sin datos del sistema**; todo desde cero; **varios inicios** posibles (02, 14). |
| 44 | Conocimiento del sistema | **Desconocido → Sospecha → Observado → Relevado → Explorado** (02, 04, 22). |
| 45 | Diseños de fábrica | **Prefabricados genéricos y subóptimos (~70–80%)**, desbloqueados con tecnología, clonables (02, 07, 22). |
| 46 | Fase 1 sin combate | Sin amenazas de naves en el sistema local; presiones de gestión; **objetivos sugeridos opcionales**; sin duración fija (02). |
| 47 | Ritmo del mundo | **Activo / Lento / Relajado** (02, 01). |
| 48 | Banda de equilibrio | **IA hasta ±50% del jugador**, techo de sistemas, no retroceden, desactivable (02, 14). |
| 49 | Primer contacto transparente | **Protocolo de paz**, regalos, objetivo común, relación visible, idiomas por fases (02, 10, 20). |
| 50 | Investigación | **Proyectos con laboratorios y científicos**: `PI_mes = 10 × (0,7 + 0,1 × habilidad)` por laboratorio, con rendimientos decrecientes (0,93^n) (13). |
| 51 | Árbol tecnológico | **8 campos × 4 eras**, prerrequisitos visibles, **sin azar**; Fase 1 con **60 tecnologías** (13). |
| 52 | Fuentes de tecnología | **Ingeniería inversa**, **intercambio y espionaje**, **compra/licencia a empresas privadas** (13, 10, 20). |
| 53 | Empresas privadas aeroespaciales | Desde **ADM-1.1** desarrollan tecnología y naves **civiles**; el jugador compra naves, alquila servicios o licencia tecnología (07, 13, 03). |
| 54 | Programas y contratos | **Órdenes masivas** (desde **ADM-2.1**) por objetivo + presupuesto + modo (propio / contratado / mixto) (07). |
| 55 | **Lenguaje y motor** | **Núcleo de simulación en Rust** (para optimización) como crate independiente del motor, con **Godot 4 + godot-rust (gdext)** para la interfaz (21). *Reemplaza la propuesta anterior de C#.* |
| 56 | Equipo y dedicación | **Un desarrollador a tiempo completo**; **Claude escribe el código** de los módulos a partir del GDD; el desarrollador revisa, prueba y ajusta el feeling (21). |
| 57 | Alcance del MVP | **Fase 1 jugable hasta cruzar el primer punto de salto**; sin combate, espionaje, mercados grises ni multi-especie; **MVP+ opcional:** primer contacto con una IA (21, 22). |
| 58 | Arquitectura | Simulación **determinista, dirigida por datos (JSON), con comandos** del jugador, planificador multi-frecuencia, guardado versionado, CLI sin interfaz, *autoplayer* y telemetría; puente gdext fino (21). |
| 59 | **Sistema del MVP** | **Sistema Solar real + elementos ficticios**: 21 cuerpos con datos aproximados, 2 asteroides ficticios con un exótico (cristales de presión) y **un punto de salto oculto a 14 UA**; órbitas circulares (22). |
| 60 | **Tierra sin bloques** | **En el MVP la Tierra es un único nodo** (2.500 M de habitantes, estratos y grupos de interés) **sin regiones ni bloques regionales**; las regiones abstractas se reservan para colonias y después (22, 05, 02). |
| 61 | **Bienes del MVP** | **16 bienes** (subconjunto de 18) con recetas simplificadas, mercado terrestre (compra ×1,15, venta ×0,7) y costo de lanzamiento como cuello de botella (22). |
| 62 | **Ruta tecnológica del MVP** | **38 tecnologías** (+3 iniciales) hasta el cruce del punto de salto, ≈ 229.000 PI (≈ 25–30 años de juego); el cierre del MVP es **cruzar el punto de salto con EXP-2** (22). |
| 63 | **Propagación de documentación** | Los docs 00–16 se actualizaron a **v0.3** con las decisiones 43–62: premisa reconciliada con el inicio terrestre (00), ritmo del mundo (01), conocimiento y Fase 1 (02, 04), costo de lanzamiento (03), Tierra única (05, 12, 14), líderes y rasgos (06, 09, 10, 20), idiomas y disposición base por especie (08), catálogo de inicios (14), UI de programas y relación visible (16), hitos de crónica (15). |

---

## Combate dinámico (resumen; detalle y números en 11)

- **Alertas de contacto** anticipadas; objetivos de aviso: colonias ≥ 12 h, capital ≥ 3 días.
- **Doctrinas y ROE** permanentes por flota.
- **Ralentización automática y ventanas de decisión.**
- **Resolución rápida** de combates desiguales (≥ 4:1).
- **Defensa preventiva:** puestos de vigilancia, patrullas, sensores, minas y **nivel de seguridad** por zona.
- **Amenazas con origen:** facciones, piratas con bases, amenazas precursoras, rebeldías internas, incursiones por puntos de salto desconocidos. En la Fase 1 no hay amenazas de naves en el sistema local (46).

---

## Riesgos principales

| Riesgo | Descripción | Mitigación |
|---|---|---|
| **Alcance** | "Aurora con más mecánicas" + economía mixta + diplomacia con líderes + mercados grises es enorme | Vertical slice temprano; lista de excluidos del MVP (21) como regla |
| **Rendimiento** | Mundo vivo, muchas naves, empresas y pops | Núcleo en Rust; LOD de simulación (01); índices sectoriales (18); presupuesto de 1 año < 0,75 s (21) |
| **Legibilidad** | Complejidad incomprensible | Herramientas de inspección desde el primer prototipo (16); relación visible con tendencia |
| **Calidad de la IA** | Facciones, líderes, empresas o piratas que actúan de forma estúpida | IA por utilidad + simulaciones sin jugador |
| **Balance económico** | Una vía domina sobre las demás | Prototipo de economía, CLI y *autoplayer* por estilo (03, 18, 19, 21) |
| **Balance de incentivos** | Bajar impuestos siempre óptimo | Retraso, límite de capital, sobreoferta y quiebras (19) |
| **Balance del contrabando y las sustancias** | "Mina de oro" sin riesgo | Prima vs. detección, consecuencias, corrupción (19) |
| **Balance del sistema de líderes** | Espionaje y golpes "solución mágica" | Riesgo real, casus belli, martirio (10, 20) |
| **Balance de seguridad y costo** | Patrullar trivial o prohibitivo | Calibrar costos y pesos (11) |
| **Bola de nieve** | El jugador gana y se aburre | Capacidad administrativa, coaliciones, guerras civiles (02, 09, 10) |
| **Late-game vacío** | Fase 3 sin desafío | Crisis internas, secesiones, contenido precursor |
| **Coherencia agregado vs. táctico** | Resultados distintos según el nivel de detalle | Calibración automática (01, 11) |
| **Combate dinámico vs. control** | Automatización que quita control | Doctrinas, ventanas, modo detallado |
| **Mantenimiento de puntos de salto propios** | Triviales o un lastre | Prototipar costos |
| **Contenido sensible** | Servidumbre, sustancias, edad laboral baja | Tratamiento sobrio y abstracto; opción de deshabilitar (14, 19) |
| **Ruido de rasgos y líderes** | Demasiada aleatoriedad en diplomacia | Desglose "¿por qué?"; niveles de conocimiento; inercia por régimen |
| **Rubber-banding percibido** | La banda de equilibrio parece trampa | Banda sorteada por facción, sin retroceso, desactivable (02) |
| **Fase 1 aburrida** | Sin combate ni amenazas puede sentirse lenta | Presiones de gestión, objetivos sugeridos, hitos frecuentes, ritmo del mundo activo |
| **Contenido de la Fase 1** | 38 tecnologías y 15 diseños de fábrica siguen siendo mucho contenido para un MVP | Priorizar el orden por hitos de 21; el resto se recorta si hace falta |
| **Duración de la investigación y del lanzamiento** | ~229.000 PI y costos de lanzamiento altos pueden dar una Fase 1 demasiado larga o lenta | Calibrar PI/mes, presupuesto y costos con la CLI (22) |
| **Programas opacos** | Las órdenes masivas pueden ocultar por qué algo falla | Informes de cuellos de botella y alertas explicadas (07, 16) |
| **Desarrollo en solitario** | Crecer el alcance, UI más cara, código generado inconsistente, fatiga | Hitos cortos y verificables, pruebas obligatorias, UI mínima, primer ciclo jugable temprano (21) |
| **gdext y curva de Rust** | La integración con Godot es más joven que C#; Rust exige disciplina con estructuras de simulación | Puente fino, núcleo y CLI independientes, versiones fijadas, IDs y arenas (21) |
| **Deriva de documentación** | Con 23 docs, las decisiones nuevas dejan secciones viejas | Registrar cada decisión en 17 y revisar las referencias cruzadas al cerrar cada ronda de diseño |

---

## Preguntas abiertas consolidadas

### Simulación
1. ¿Cuántos sistemas pueden estar en foco a la vez? (01)
2. ¿"Indiferente" o "Cronista" por defecto? Alcance y costo de los relés. (01)
3. Ritmo real de cada fase y del modo "Lento" tras los primeros prototipos. (02)

### Economía y logística
4. ¿Cuántas empresas privadas se simulan individualmente? ¿Mercado de acciones? (03)
5. Calibración de precios base, recetas, β sectoriales, márgenes y crecimiento de privadas. (18, 22)
6. Canasta básica por especie. (05, 08, 18)
7. Restricciones de tránsito por puntos de salto. (04)
8. Costos, alcance y escalado de los puntos de salto propios. (04, 13)
9. Valores de prima, `p_detección` y comisión del contrabando; ¿zonas francas como facciones o instalaciones? (19)
10. Costo en capital político de cada cambio de ley o de impuesto por régimen. (19)
11. Modelo de dependencia por pop. (19)
12. Reparto de tareas entre propio y contratado en programas mixtos. (07)
13. **Costo de lanzamiento vs. presupuesto** (¿~20 lanzamientos/año al inicio es el ritmo deseado?). (22)

### Población, personajes y especies
14. Cantidad de regiones por cuerpo para después del MVP (en el MVP, Tierra única). (05, 14, 22)
15. Habilidades de notables (~15–20) y máximo de notables por facción. (06)
16. Confirmar las 4 especies del núcleo y cuáles son jugables; calibrar disposiciones base y factores de idioma propuestos en 08.
17. Calibrar el **apoyo público** y su relación con el gasto espacial. (05, 09, 22)

### Política y diplomacia
18. ¿La servidumbre es parámetro de partida (on/off) o ley siempre disponible? (09)
19. ¿Elecciones simuladas con **partidos** o sólo con grupos de interés? (09, 20)
20. Cláusulas de salida en acuerdos de servicios; instituciones galácticas. (10)
21. Calibrar `w`, inercias, rasgos, espionaje y `p_ataque_inicial`; ¿herencia de rasgos en linajes? (20, 02)
22. Reglas de sucesión del régimen colmena por especie. (08, 20)
23. Definir el **poder** comparable entre facciones para la banda de equilibrio. (02)
24. ¿Hay ayuda para saltar el muro de la Fase 1 a la 2? (02, 10)
25. Operación de espionaje "robo de tecnología". (13, 20)

### Combate
26. Calibrar `poder_combate` y el umbral 4:1. (11)
27. Tablas de armas, blindaje y daño por componente. (07, 11)
28. Valores de sensibilidad/firma por tecnología. (11, 13)
29. Fuerza y comportamiento de piratas por zona. (11, 19)

### Universo y tecnología
30. Tamaño de la galaxia por defecto (¿100 o 500 sistemas?). (14)
31. Distribución de puntos de salto y proporción de ocultos. (14)
32. Catálogo de inicios (cuántos además de la Tierra). (02, 14)
33. **Tecnologías de la Fase 2+**. (13)
34. Calibrar costos y PI/mes; masa/capacidad/costo de los diseños de fábrica. (13, 07, 22)
35. Cantidad y tipos de laboratorios y su mantenimiento; ¿se agrega ADM-0.2 al MVP? (13, 22)
36. Lunas de Júpiter y Titán: ¿sólo observables en el MVP o se esconden hasta el MVP+? (22)
37. Probabilidades base de fallos de lanzamiento y de misión. (07, 12, 22)

### Desarrollo
38. Confirmar gdext + Godot con un pequeño prototipo (versión, recarga en caliente) antes de comprometer la UI. (21)
39. Qué parte de la UI va en escenas del editor y cuál programática; estilo visual mínimo. (21)
40. ¿Hay `cargo` y acceso a crates en el entorno de Claude para compilar y probar en la conversación? (21)

---

## Hoja de ruta sugerida

### Etapa A — Documentación (cerrada para el MVP)
Docs 00–16 en v0.3 + 18, 19, 20, 21 (v0.2), 22 (v0.1). **La propagación está hecha: se puede empezar a programar el MVP (M0).** Quedan pendientes opcionales (fichas completas de especies, sólo para MVP+ y Fase 2).

### Etapa B — Prototipos y primeros hitos (CLI primero)
Siguen el plan por hitos del doc 21:
1. **Cimientos, reloj, datos y puente gdext** (M0).
2. **Mundo y órbitas** (M1).
3. **Investigación** (M2).
4. **Economía mínima** (M3).
5. **Diseño y flotas** (M4) → **primer ciclo jugable** (~4,5–5,5 meses).
Prototipos posteriores (fuera del MVP): combate con doctrinas (11), líderes y diplomacia (10, 20), malestar y guerra civil (09), mercados grises (19), generación de universo (14).

### Etapa C — MVP (Fase 1 jugable)
M5–M8 del doc 21: sociedad, privados y programas, cierre de la Fase 1 (cruce del punto de salto), pulido y balance. Estimación total **7–10 meses** (±40%).

### Etapa D
Varios sistemas, exploración, comercio interestelar, pactos, líderes y espionaje, piratas con bases, minas, anomalías, incursiones, contrabando y zonas francas.

### Etapa E
Escala de sector: política interna, secesión, diplomacia compleja, puntos de salto propios, contenido precursor.

### Ruta tecnológica del MVP (38 + 3 iniciales; detalle en 22)

| Campo | Tecnologías |
|---|---|
| PRO | 0.1, 1.1, 1.2, 2.1, 2.2, 3.2 |
| ENE | 0.1, 1.1, 1.2, 2.1 |
| MAT | 0.2, 1.1, 1.2, 2.2 |
| SEN | 0.2, 1.1, 1.2, 2.1, 2.2, 3.1, 3.2 |
| COM | 0.1, 1.1, 1.2, 2.1 |
| VID | 0.1, 1.1, 1.2, 2.1 |
| MIN | 1.1, 2.1, 2.2, 2.3 |
| ADM | 0.1, 1.1, 2.1, 2.2, 2.3 |

(Más las iniciales ya poseídas: SEN-0.1, COM-0.2, MAT-0.1.) Llega hasta el cruce del punto de salto (PRO-3.2).

---

## Próximos pasos (orden sugerido)

1. ~~Economía con números~~ → **hecho (18, 19)**.
2. ~~Combate y seguridad con números~~ → **hecho (11)**.
3. ~~Líderes, regímenes y rasgos~~ → **hecho (20)**.
4. ~~Fase 1 en detalle~~ → **hecho (02 v0.3)**.
5. ~~Tecnologías de la Fase 1 y diseños de fábrica~~ → **hecho (13 v0.3, 07 v0.3)**.
6. ~~Alcance del MVP y arquitectura técnica~~ → **hecho (21 v0.2, con Rust + gdext)**.
7. ~~Datos iniciales del MVP~~ → **hecho (22 v0.1)**.
8. ~~Propagar la Fase 1 y el inicio por defecto a los demás docs~~ → **hecho (00–16 en v0.3, decisión 63)**.
9. ~~Pendientes menores (referencias a 18, 19, 20, 22; costo de puntos de salto)~~ → **hecho**.
10. **Empezar M0** (código): workspace de Cargo, crate de simulación (`heliarca-sim`), CLI, carga de datos JSON de 22, reloj, planificador, comandos, extensión gdext mínima. Claude puede generar el código inicial; hace falta confirmar la versión de Godot y si hay `cargo` disponible (pregunta 40).
11. Fichas de especies para el MVP+ y resto de sistemas y tecnologías de la Fase 2+.
