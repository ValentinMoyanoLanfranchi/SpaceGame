# 00 — Visión general

> **Nombre en clave:** HELIARCA (provisorio)
> **Estado:** borrador v0.3 — modelo **Aurora 4X en 2D**; inicio por defecto en la **Tierra pre-espacial**; MVP definido (21, 22). Documentación de ideas y sistemas; lo decidido está registrado en `17`.

---

## Concepto en una frase

Un 4X espacial 2D por incrementos de tiempo, donde gobernás un **Estado** desde su planeta capital —por defecto la **Tierra, sin tecnología espacial ni datos del sistema**— y, si lo lográs, hasta un sector entero de la galaxia, con una **economía mixta profunda** (empresas estatales y privadas, impuestos por categoría, mercados grises), logística física, diplomacia con líderes y un universo que sigue su curso con o sin vos.

**Modelo principal: Aurora 4X.** Mantenemos su columna vertebral (sistemas estelares en 2D, puntos de salto, relevamiento, diseño detallado de naves, sensores, mantenimiento, avance por incrementos, desarrollo tecnológico desde cero) y le sumamos: economía con mercados y empresas, población y política más profundas, un mundo con facciones autónomas, historia emergente, diseños de fábrica para principiantes, programas de órdenes masivas y un combate más informado y dinámico.

---

## Pilares de diseño

1. **El mundo no te espera.** Facciones, mercados, guerras, piratas y catástrofes ocurren independientemente del jugador. No existen eventos "dirigidos al jugador". Toda nave hostil viene de alguna parte. (El **ritmo del mundo** es configurable: se puede jugar más relajado, ver 02.)
2. **Todo existe en algún lugar.** No hay recursos globales abstractos. Cada tonelada de mineral, cada nave y cada crédito está en un lugar físico y tiene que moverse. La logística es la columna vertebral.
3. **La escala se gana.** Empezás con un planeta y nada de tecnología espacial. Llegar a otros cuerpos, a otros sistemas y a gobernar un sector exige resolver problemas reales (industria, tecnología, logística, administración), no un botón de desbloqueo.
4. **La complejidad se delega.** Al crecer, los problemas anteriores no desaparecen: se delegan en gobernadores, empresas estatales, **programas de órdenes masivas**, almirantes con doctrinas, patrullas y puestos automáticos. Las instituciones pueden fallar.
5. **Muchos caminos económicos.** Se puede jugar todo-estatal, capitalismo puro, imperialismo (conquistar y saquear conocimiento, recursos y tecnología), cliente de una potencia, construir todo desde cero, o jugar en la zona gris (sustancias, contrabando) con sus riesgos.
6. **Historia emergente.** Las mejores historias salen de la interacción entre sistemas. El juego registra y cuenta lo que pasó.
7. **Profundo pero legible e informado.** Toda complejidad es inspeccionable ("¿por qué pasó esto?"), y el jugador **nunca queda atrapado sin información ni opciones**: alertas anticipadas, ventanas de decisión, relaciones diplomáticas visibles y automatización con reglas.

---

## Qué tomamos de cada referente

| Juego | Qué tomamos | Documentos |
|---|---|---|
| **Aurora 4X** *(principal)* | Sistemas en 2D con órbitas en rieles; puntos de salto; relevamiento geológico y gravimétrico; yacimientos con cantidad y accesibilidad; diseño detallado de naves; sensores y detección; mantenimiento y combustible; avance por incrementos; investigación con laboratorios y científicos; desarrollo desde cero | 02, 04, 07, 11, 13, 14 |
| **X4: Foundations** | Economía con naves reales que compran, transportan y venden; empresas y cadenas de producción; facciones que guerrean por su cuenta | 03, 04, 18 |
| **Victoria 3** | Pops, mercados, grupos de interés, leyes, política económica, leyes de trabajo | 03, 05, 09, 19 |
| **Europa Universalis IV** | Relaciones y tratados, coaliciones, casus belli | 10 |
| **Crusader Kings 3** | Líderes con rasgos y ambiciones, sucesión, intriga, golpes y asesinatos | 06, 09, 10, 20 |
| **Stellaris** | Variedad de especies y gobiernos, primer contacto, crisis tardías | 08, 10 |
| **Dwarf Fortress** | Historia del mundo generada antes de empezar; leyendas y objetos con historia | 14, 15 |
| **Factorio** | Cadenas productivas | 07, 18 |
| **Frostpunk** | Leyes de emergencia en crisis | 09 |
| **RimWorld / Kenshi** *(secundarios)* | Sustancias con ventajas y desventajas, rasgos de personajes, mundo indiferente al jugador, narrador opcional | 01, 06, 19 |

---

## Fantasía del jugador por etapa

- **Inicio:** "Tengo un planeta entero y no sé qué hay allá arriba. Tengo que mirar el cielo, aprender a llegar y decidir adónde ir."
- **Medio:** "Mi civilización comercia, se expande y se defiende. Tengo socios y enemigos."
- **Final:** "Gobierno algo más grande que yo, y lucho para que no se desarme."

---

## Rol del jugador

El jugador controla un **Estado**. No hay avatar: los personajes (gobernadores, almirantes, científicos, directores) son **notables** que el jugador gestiona y que tienen voluntad propia (ver 06). Cada facción, incluida la del jugador, tiene un **líder** cuyas cualidades afectan su comportamiento (10, 20).

### Régimen inicial y estilos de juego
Al empezar se elige el **régimen** (modelo político y económico) y se puede reformar después con costo político (20). Estilos de juego soportados (no excluyentes):

| Estilo | Idea |
|---|---|
| **Estatal** | El Estado dirige la industria y el comercio; empresas privadas mínimas o prohibidas |
| **Mixto** | Empresas estatales en rubros estratégicos + empresas privadas en el resto, reguladas e impuestas |
| **Capitalista** | Estado chico, mercado libre, crecimiento por volumen de comercio y recaudación |
| **Imperialista** | Conquistar otras civilizaciones para obtener recursos, tecnología y conocimiento |
| **Cliente / protectorado** | Pagar a una potencia por protección, recursos, tecnología o desarrollo, a cambio de contraprestaciones |
| **Autárquico** | Hacerlo todo uno mismo, desde cero |

Además, el jugador puede usar **palancas finas** como impuestos por categoría, incentivos, y —si el mundo lo permite— mercados grises y leyes de trabajo duras (19). Desde mediados de la Fase 1 puede además **delegar en programas y contratos** con empresas privadas (07).

### Escenarios de inicio

| Inicio | Descripción | Estado |
|---|---|---|
| **Tierra — era pre-espacial** *(por defecto)* | Planeta capital unificado, 2–3 mil millones de habitantes, sin tecnología espacial y sin datos del sistema | Definido (02, 22) |
| Era espacial temprana | Ya hay satélites y una estación; sistema parcialmente relevado | Propuesta |
| Mundo de otra especie | El mismo arranque con una especie no humana (08) | Propuesta |
| **Civilización caída** | Tecnología avanzada pero base industrial reducida; premisa original de Diáspora y Colapso | Propuesta |
| Estado cliente | Arranca dependiente de otra potencia (más ayuda, más deudas y exigencias) | Propuesta |
| Modo observador / crónica | Sin facción, sólo mirar el mundo | Propuesta |

### Derrota
Si el Estado pierde su capital o se desintegra: continuar con un remanente (exilio, colonia superviviente) o seguir en **modo crónica**. No hay "game over" forzado.

---

## Escalas y fases (detalle en `02`)

| Fase | Escala | Problema central |
|---|---|---|
| 1 — Sistema local | Capital, lunas, planetas vecinos, asteroides | Desarrollar tecnología espacial desde cero, relevar el sistema, industria fuera del planeta, pozo gravitatorio, apoyo público, descubrir el FTL |
| 2 — Interestelar | Varios sistemas conectados por puntos de salto | Exploración, colonización, primer contacto, cuellos de botella, seguridad |
| 3 — Sector | Decenas de sistemas | Capacidad administrativa, secesión, gran política |

---

## Loops de juego

- **Micro (minutos):** diseñar, construir, dar órdenes a flotas, resolver alertas y ventanas de decisión.
- **Meso (horas):** abrir rutas, instalar colonias, armar cadenas productivas, definir programas y contratos, ajustar impuestos e incentivos, negociar, manejar crisis, patrullar.
- **Macro (decenas de horas):** reformar el régimen, guerras largas, sobrevivir a secesiones, gobernar un sector.

## Tiempo

**Incrementos puros estilo Aurora** (5 s · 1 h · 1 día · 5 días · 30 días), sin tiempo real, con pausa total. El avance se interrumpe ante eventos que el jugador marca como relevantes, y el juego puede **ralentizarse solo** ante contactos hostiles para dar ventanas de decisión (ver 11, 16).

---

## Tono y ambientación

Ciencia ficción **"dura-ish"**: física plausible en lo cotidiano (combustible, calor, radiación, distancias) con licencias controladas (puntos de salto, tecnología precursora). Tono sobrio, a veces brutal, con humor emergente. Los temas delicados (servidumbre, sustancias, trabajo infantil) se tratan de forma **abstracta y sobria**, con consecuencias, y son **opcionales** en la configuración del mundo.

### Premisa: La Diáspora y el Colapso (adaptada)
Hace siglos una civilización muy avanzada sembró la galaxia de colonias y luego colapsó. Los planetas quedaron aislados, la tecnología se perdió, el viaje interestelar se olvidó y quedaron ruinas.

**Cómo encaja con el inicio por defecto:** en el inicio en la **Tierra pre-espacial** el jugador **no es heredero** de esa civilización: es una civilización joven que descubre el espacio. La premisa describe **el mundo que encuentra**: ruinas y restos precursores, civilizaciones caídas o aisladas, puntos de salto olvidados y conflictos heredados. El inicio **"Civilización caída"** conserva la premisa original (un superviviente que perdió el FTL).

Esto justifica: mundo poblado e historia previa, contenido tardío (ruinas, artefactos, secretos) y otras razas con las que comerciar o guerrear.

---

## Fuera de alcance (por ahora)

- Multijugador.
- Control directo de individuos o combate en primera persona.
- Mapas de superficie planetaria jugables.
- Campaña con historia guionada.
- Tiempo real continuo.

El **MVP** (21, 22) es la Fase 1 jugable hasta cruzar el primer punto de salto; combate, espionaje, mercados grises, varias especies y diplomacia con líderes quedan para después.

---

## Tecnología de desarrollo

**Rust** para el núcleo de simulación (determinista, dirigido por datos) y **Godot 4 con godot-rust (gdext)** para la interfaz, por un único desarrollador con ayuda de Claude para el código (21).

---

## Índice de documentos

| # | Archivo | Sistema |
|---|---|---|
| 00 | `00_vision_general.md` | Visión, pilares, referentes |
| 01 | `01_simulacion_del_mundo.md` | Mundo vivo, niveles de detalle, IA, información, ritmo del mundo |
| 02 | `02_progresion_y_escalas.md` | Fases, inicios, Fase 1 en detalle, banda de equilibrio, primer contacto, capacidad administrativa |
| 03 | `03_economia.md` | Mercados, empresas estatales y privadas, comercio, finanzas |
| 04 | `04_recursos_y_logistica.md` | Yacimientos, relevamiento, transporte, puntos de salto |
| 05 | `05_colonias_y_poblacion.md` | Pops, necesidades, regiones, malestar |
| 06 | `06_personajes.md` | Notables y líderes |
| 07 | `07_produccion_y_diseno.md` | Cadenas, materiales, diseño, diseños de fábrica, programas y contratos |
| 08 | `08_especies_y_culturas.md` | Especies, culturas, religiones, idiomas |
| 09 | `09_facciones_y_politica_interna.md` | Régimen, grupos de interés, servidumbre, guerras civiles |
| 10 | `10_diplomacia.md` | Relaciones, líderes, primer contacto, tratados, acuerdos de servicios, espionaje |
| 11 | `11_guerra_y_combate.md` | Combate espacial, seguridad, terrestre, estratégico |
| 12 | `12_entorno_y_catastrofes.md` | Planetas, clima, catástrofes, ecología |
| 13 | `13_tecnologia_e_investigacion.md` | Investigación por proyectos, 60 tecnologías de la Fase 1, puntos de salto propios |
| 14 | `14_universo_y_generacion.md` | Inicios, generación del universo, de la historia y de amenazas |
| 15 | `15_historia_emergente.md` | Crónicas, leyendas, rumores |
| 16 | `16_interfaz_y_legibilidad.md` | UI, alertas, herramientas de información, modding |
| 17 | `17_preguntas_abiertas_y_roadmap.md` | Decisiones, riesgos, hoja de ruta |
| 18 | `18_bienes_cadenas_y_parametros_economicos.md` | Bienes (~73), bienes de cambio, cadenas, parámetros de mercado |
| 19 | `19_impuestos_incentivos_y_mercados_grises.md` | Impuestos por categoría, incentivos, sustancias, contrabando, leyes de trabajo |
| 20 | `20_lideres_regimenes_y_rasgos.md` | Rasgos de líder, 7 regímenes, sucesión, conocimiento de líderes, espionaje |
| 21 | `21_alcance_mvp_y_arquitectura.md` | Alcance del MVP, arquitectura en Rust + Godot, plan por hitos |
| 22 | `22_datos_iniciales_mvp.md` | Sistema solar, bienes, instalaciones, tecnologías, diseños y Tierra del MVP |

---

## Glosario

- **Pop:** grupo de habitantes que comparten especie, cultura, clase, profesión y región. Unidad demográfica agregada.
- **Notable:** personaje simulado con detalle (gobernador, almirante, científico, director, diplomático, espía, magnate, líder).
- **Región:** subdivisión abstracta de un planeta o luna con recursos, población e infraestructura propias. En el MVP la Tierra es un único nodo sin regiones.
- **Nodo de mercado:** lugar con inventario y precios propios (colonia, estación).
- **Bienes de cambio:** bienes que circulan sobre todo entre privados (bebidas, joyas, tecnología de consumo, sustancias…), agrupados en sectores con su propio índice de precios.
- **Empresa estatal / privada:** entidad productiva controlada por el Estado o por inversores; las privadas operan por rentabilidad.
- **Capacidad administrativa:** cuánto territorio/población puede gobernar una facción sin perder eficiencia.
- **Relevancia:** cuánto le importa el jugador (u otra facción) a una facción dada.
- **Punto de salto:** conexión FTL entre sistemas; natural, o construido por una facción con tecnología avanzada y mantenimiento.
- **Nivel de seguridad:** valor por sistema/zona calculado según sensores, patrullas, defensas y minas.
- **Doctrina / ROE:** reglas permanentes de una flota (cuándo atacar, retirarse, pedir ayuda).
- **Anomalía:** señal detectada de origen incierto (posible incursión por un punto de salto desconocido, fenómeno natural, etc.).
- **Zona franca:** estación o punto fuera de jurisdicción donde todo es legal; hub de mercado negro.
- **Malestar (unrest):** nivel de descontento de una población, con escalada hacia la rebelión.
- **PI (puntos de investigación):** unidad de costo y de producción de la investigación (13).
- **Estado de conocimiento:** Desconocido → Sospecha → Observado → Relevado → Explorado (02).
- **Diseño de fábrica:** diseño prefabricado y subóptimo, pensado para principiantes (07).
- **Programa:** orden masiva (minería, colonización, relevamiento, transporte) ejecutada por flota propia, empresas privadas o ambas (07).
- **Ritmo del mundo:** cuán rápido avanzan las otras civilizaciones (Activo / Lento / Relajado, 02).
- **Banda de equilibrio:** límite al poder de las IA respecto del jugador (02).
- **Rasgo (de líder):** cualidad personal con efectos numéricos; los líderes tienen 2–4 (20).
- **Peso del líder:** cuánto influye el líder en la política exterior según el régimen (10, 20).
