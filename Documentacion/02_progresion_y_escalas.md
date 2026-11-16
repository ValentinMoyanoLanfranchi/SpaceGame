# 02 — Progresión y escalas

> v0.4 · Fase 1 detallada: arranque desde la Tierra sin conocimiento del sistema, tecnología desde cero, diseños de fábrica, ritmo del mundo y banda de equilibrio de las IA, primer contacto transparente. En el MVP la Tierra es un único nodo (sin regiones). Los números son iniciales; los datos concretos del MVP están en 22.

## Filosofía

La progresión no es un árbol de desbloqueos: es **cambiar de escala de problemas**. Cada salto exige resolver condiciones concretas y trae problemas nuevos sin eliminar los anteriores (se delegan). **El mundo también progresa**: otras facciones llegan al espacio antes o después que vos, y la frontera se mueve (pero dentro de límites, ver "Ritmo del mundo y banda de equilibrio").

La progresión tampoco es irreversible: una guerra civil, un síndrome de Kessler, la pérdida de conocimiento o la caída de la capital pueden hacerte **retroceder** de fase.

---

## Inicios (escenarios de partida)

El juego admite **varios inicios**; cada uno define población, tecnología, conocimiento del sistema y recursos.

| Inicio | Descripción | Estado |
|---|---|---|
| **Tierra — Era pre-espacial (default)** | Planeta capital unificado, **2–3 mil millones de habitantes**, **sin tecnología espacial**, **sin datos del sistema solar** (sólo pistas de telescopios). Todo se desarrolla desde cero, como en Aurora. | **Definido** (datos en 22) |
| Era espacial temprana | Ya hay satélites y una estación; el sistema está parcialmente relevado. | Propuesta |
| Mundo de otra especie | El mismo arranque con una especie no humana (08). | Propuesta |
| Civilización caída | Tecnología avanzada pero base industrial reducida (premisa original de Diáspora y Colapso, 00). | Propuesta |

El resto del documento describe el inicio por defecto.

---

## Fase 1 — Sistema local

### Estado inicial (inicio por defecto)

- **Planeta capital unificado** bajo el gobierno del jugador, con **2–3 mil millones de habitantes** (pops agregados). **En el MVP la Tierra es un único nodo, sin regiones ni bloques** (05, 22); las regiones abstractas se reservan para colonias grandes y para después.
- **Sin naves, sin estaciones, sin tecnología espacial.** Hay industria, ciencia y telecomunicaciones de nivel terrestre. Todo lo espacial se **investiga y construye**.
- **Sin datos del sistema.** El jugador no sabe qué cuerpos existen ni sus recursos. Sólo hay **pistas**: telescopios y observatorios sugieren la existencia de cuerpos cercanos, con **datos imprecisos** (rangos amplios de masa, órbita y composición; ver "Conocimiento del sistema").
- **Presión sobre el Estado:** escasez de recursos terrestres, costo político del programa espacial frente al gasto social y tensiones internas (ver "Presiones").
- **No hay amenazas de naves en el sistema local.** Los piratas, las incursiones y los rivales armados pertenecen a la Fase 2.

### Conocimiento del sistema

Cada cuerpo, punto de salto y recurso tiene un **estado de conocimiento**:

| Estado | Qué sabés | Cómo se logra |
|---|---|---|
| **Desconocido** | Nada | — |
| **Sospecha** | Existencia probable; datos con rangos amplios | Telescopios terrestres, observatorios, análisis de perturbaciones |
| **Observado** | Órbita y masa aproximada; composición superficial parcial | Telescopios orbitales, sobrevuelo de sondas |
| **Relevado** | Datos precisos de yacimientos y condiciones (cap. 04) | Sondas orbitales, relevamiento geológico y gravimétrico |
| **Explorado** | Detalle fino de regiones y sitios; bonus a colonización | Aterrizaje, robots, presencia humana |

- Los **telescopios** mejoran con la tecnología: más resolución y alcance (poder de telescopio contra dificultad del cuerpo, 22). Dan pistas, no certezas; un cuerpo "sospechado" puede resultar distinto a lo esperado.
- El **relevamiento gravimétrico** (04) es la vía para localizar **puntos de salto**; se hace **por zonas** del sistema (22).
- Un estado de conocimiento sólo mejora por **acciones** del jugador (investigación, sondas, misiones), no por el paso del tiempo.

### Tecnología desde cero

El camino típico de la Fase 1 (detalle en 13) pasa por eras:

| Era | Hitos típicos | Habilita |
|---|---|---|
| **0 — Pre-orbital** | Cohetes químicos, telescopios mejores, satélites de observación y comunicaciones | Primeros datos del sistema |
| **1 — Órbita** | Estaciones orbitales, sondas robóticas, telescopios orbitales, fisión compacta | Relevamiento de lunas y planetas vecinos, primeras bases |
| **2 — Expansión local** | Minería de asteroides y lunas, combustible, naves de transporte, colonias dependientes | Economía fuera del planeta |
| **3 — Exploración profunda** | Propulsión mejorada, sensores avanzados, relevamiento gravimétrico, sistemas de salto | Punto de salto, salida a la Fase 2 |

Cada tecnología tiene **costo, tiempo y requisitos**; no se saltan eras. Las IA siguen caminos similares con perfiles propios.

### Diseños de fábrica

Para ayudar a principiantes, el juego incluye **diseños prefabricados** de naves, estaciones, satélites, sondas y drones (07):

- Cubren los roles básicos (transporte, exploración, minería, relevamiento, estación pequeña, satélite de comunicaciones, etc.) para cada nivel tecnológico.
- Son **genéricos y subóptimos** a propósito: rinden aproximadamente **70–80%** de un diseño optimizado con la misma tecnología (más masa muerta, menos eficiencia de combustible, sin especialización).
- Se **desbloquean con la tecnología** correspondiente; no son gratis.
- Se pueden **clonar y editar** para crear diseños propios (07).
- Los prefabricados **no envejecen automáticamente**: al llegar una tecnología nueva aparece una nueva versión de fábrica, también subóptima.

### Presiones de la Fase 1

La Fase 1 es una etapa de **exploración y construcción de base**, no de combate. Las presiones son de **gestión**:

| Presión | Descripción |
|---|---|
| **Escasez de recursos** | Metales, agua, combustible o energía son cuellos de botella casi permanentes; fuerzan decisiones sobre qué explotar y dónde (04, 18) |
| **Costo del programa espacial** | El gasto en investigación y construcción compite con el gasto social; si es excesivo baja el **apoyo público** (05, 09, 22). El **costo de lanzamiento** domina lo que se puede subir a órbita |
| **Tensión social** | Grupos de interés, malestar por desigualdad, nacionalismos y sindicatos presionan por prioridades distintas (09) |
| **Capacidad administrativa** | Más colonias y más empresas consumen capacidad (ver más abajo) |
| **Ritmo del mundo** | Las otras civilizaciones avanzan (ver más abajo) |

### Qué hace el jugador

1. **Investigar y construir** la base espacial inicial (satélites, sondas, estación).
2. **Relevar el sistema** y decidir **qué cuerpos conviene colonizar**, según recursos, facilidad de colonización y posición.
3. **Fundar colonias** (dependientes al principio) y montar la economía fuera del planeta, con **empresas estatales y privadas** (03).
4. **Desarrollar naves de exploración** cada vez más capaces para ir más lejos.
5. **Delegar** con **programas y contratos** cuando hay muchas naves (07).
6. **Mantener el equilibrio interno**: apoyo público, empresas, grupos de interés.
7. **Descubrir el primer punto de salto** y prepararse para cruzarlo.

### Objetivos sugeridos (opcionales)

El juego ofrece una **lista de objetivos sugeridos** que se pueden ignorar, sin tutorial obligatorio:

| Objetivo | Efecto |
|---|---|
| Lanzar el primer satélite | Datos y comunicaciones |
| Enviar la primera sonda | Relevamiento inicial |
| Alcanzar la órbita con una estación | Base para misiones |
| Relevar la Luna / el planeta más cercano | Conocimiento y recursos |
| Fundar la primera colonia fuera del planeta | Economía extraplanetaria |
| Iniciar la minería de asteroides | Metales y combustible |
| Localizar un punto de salto | Condición para la Fase 2 |
| Cruzar un punto de salto | Fin de la Fase 1 |

Se acompañan de **ayudas contextuales** y de la UI de legibilidad (16); no hay misiones obligatorias.

### Condición de salida → Fase 2

Con **tecnología de FTL** (13): localizar un punto de salto, tener una nave capaz de cruzarlo y capacidad para sostener operaciones del otro lado. **No hay duración fija**: el ritmo lo marca el jugador y, si está activado, el **ritmo del mundo**.

---

## Ritmo del mundo y banda de equilibrio

### Ritmo del mundo (configurable)

Las demás civilizaciones **siguen desarrollando su poder espacial** mientras el jugador avanza. Al crear la partida se elige:

| Modo | Efecto |
|---|---|
| **Activo (default)** | Las IA avanzan a su ritmo; si el jugador se demora, otros toman los mejores sistemas |
| **Lento** | Las IA avanzan con ritmo reducido (≈ ×0,5) |
| **Relajado** | El avance de las IA se **desactiva** hasta el primer contacto (modo para jugar tranquilo) |

También se puede cambiar durante la partida (con una marca en la crónica). Ver 01.

### Banda de equilibrio (límite a las IA)

Para evitar enfrentarse a imperios imposibles, el crecimiento de las IA está **limitado en relación con el jugador**:

```
poder(facción) = f(sistemas, población, economía, tecnología, flota)
poder_techo(facción) = poder_jugador × (1 + banda_facción)
banda_facción ∈ [−0,5; +0,5]  (se sortea al crear la partida; default hasta +50% / −50%)
```

- Cada facción **relevante** (con posibilidad de contactarte) tiene su **banda**: algunas serán más débiles que el jugador y otras hasta **35–50% más fuertes**.
- La IA **no retrocede** por el techo: si lo alcanza, su crecimiento se **frena** (se estanca), pero conserva lo que tiene.
- **Límite de expansión:** una facción no puede tener muchos más sistemas que el jugador (ej.: con 1 sistema, ninguna IA aparece con 30). El techo de sistemas es aproximadamente `ceil(sistemas_jugador × (1 + banda)) + margen_inicial`.
- **Coaliciones:** como el rival más fuerte llega a ×1,5, juntar aliados y atacar en bloque es **viable y necesario** (10).
- Se puede **desactivar** la banda (modo "sin límites") para partidas difíciles.
- Las facciones lejanas (otro sector, sin posibilidad de contacto en la fase actual) pueden evolucionar libremente hasta que entren en el radio de contacto.
- El **poder** comparable entre facciones (pesos de sistemas, población, economía, tecnología, flota) se define con prototipos (17).

---

## Primer contacto transparente

A diferencia de Aurora, donde no está claro cuándo mejoran o empeoran las relaciones, aquí el contacto es **legible** (16, 10):

- **Protocolo de paz:** el jugador puede **desactivar armas** (postura "no hostil") y **activar su transmisor** (señal de paz). Es una orden de flota/estación. (ADM-2.3 lo desbloquea, 13.)
- **Facciones neutrales** que reciben esa señal **no atacan** (salvo líderes muy agresivos) y se abre el canal de comunicación.
- **Regalos y gestos:** enviar recursos, datos o tecnología modesta mejora la actitud.
- **Objetivo común:** una vez establecida la comunicación, ambas partes pueden **fijar un objetivo** (comercio, alianza defensiva, exploración conjunta, no agresión).
- **Relación visible:** un indicador con **tendencia** (sube/baja) y su **desglose** ("¿por qué?"), alimentado por especie, régimen, líder y agravios (10, 20).
- **Idiomas:** al principio no hay lenguaje común. El entendimiento avanza por fases (factores por especie en 08):

| Fase de comunicación | Qué se puede hacer | Cómo avanza |
|---|---|---|
| **Sin lenguaje** | Gestos y señales básicas: estado de ánimo (amistoso / hostil / ofrenda), intercambio simple | Tiempo de contacto |
| **Traducción parcial** | Mensajes simples, comercio básico, acuerdos sencillos | Notables con xenología (COM-1.2), intercambio de datos (≈ 1–3 años, según especie) |
| **Lenguaje común** | Diplomacia completa | Continúa el estudio |

Riesgo inicial de hostilidad (valores iniciales):

```
p_ataque_inicial = clamp( 0,15 + 0,12×Ag_postura − 0,10×Ap_postura − 0,25×señal_de_paz, 0,02, 0,70 )
```

`Ag_postura` y `Ap_postura` son los ejes de la postura del líder hacia vos (−3 a +3, 20). La señal de paz **reduce el riesgo, no lo elimina**: un líder muy agresivo o xenófobo puede ignorarla.

---

## Fase 2 — Interestelar

- **Escala:** varios sistemas unidos por puntos de salto.
- **Problemas:**
  - **Exploración y relevamiento:** descubrir sistemas y puntos de salto (algunos ocultos).
  - Colonias distantes que desarrollan identidad propia (05).
  - **Primer contacto** con otras especies y civilizaciones, **con sus líderes** (10, 20); pactos comerciales y diplomáticos.
  - **Seguridad:** puntos de salto como cuellos de botella; patrullas, puestos de vigilancia, minas, niveles de seguridad por zona (11).
  - **Piratas con bases** y rutas comerciales que proteger (aparecen en esta fase).
  - Primeras **incursiones por puntos de salto desconocidos** (dificultad opcional).
  - Construcción de **puntos de salto propios** (13).
  - Comercio y empresas privadas operando entre sistemas (03).
- **Sistemas protagonistas:** exploración (14), red de puntos de salto, gobernadores de sistema, flotas con doctrinas, diplomacia interestelar con líderes, economía mixta.
- **Condición de salida → Fase 3:** capacidad administrativa suficiente (instituciones, burocracia, capitales regionales, comunicaciones rápidas).

## Fase 3 — Sector

- **Escala:** agrupaciones de 10–30 sistemas; posiblemente varios sectores.
- **Problemas:** secesión, guerras civiles, gran política entre potencias, sobreextensión, crisis de escala galáctica.
- **Sistemas protagonistas:** gobernanza delegada (virreyes, consejos), leyes de gran escala, megaproyectos, guerras en varios frentes, bloques comerciales.
- **Final abierto:** no hay fase 4 obligatoria. Late-game: crisis galácticas, misterio precursor, colapso y renacimiento.

> Las fases son una guía de progresión y de ritmo, no pantallas de bloqueo. Un jugador puede quedarse en la Fase 1 o 2 indefinidamente y es una partida válida (con el ritmo del mundo en "Relajado", sin presión externa).

---

## Mecánica central: capacidad administrativa

Freno natural a la "bola de nieve". Cada facción tiene una **capacidad administrativa** (generada por burócratas, instituciones, tecnología de comunicaciones, infraestructura, leyes y la **calidad del líder**). Cada territorio (colonia, sistema) **consume** capacidad.

Fórmula orientativa:

```
costo_admin(territorio) = base
                          × población^0.5
                          × (1 + latencia_comunicacion_dias / 30)
                          × (1 + distancia_cultural)
                          × (1 − autonomía × 0.6)
```

**Sobreextensión** (consumo > capacidad) → corrupción, ineficiencia, menor lealtad, separatismo. Dar más **autonomía** reduce el costo pero también el control (impuestos, obediencia). Las **empresas privadas** también reducen la carga administrativa a cambio de menos control directo.

---

## Delegación por escala

| Fase | Control directo | Delegado a |
|---|---|---|
| 1 | Todo lo que el jugador elija (manual al inicio) | Gobernadores de colonia con directivas; directores de empresas estatales; jefes de proyecto; **programas y contratos** (ADM-2.1) |
| 2 | Lo que el jugador elija | Gobernadores de sistema; almirantes con reglas de enfrentamiento; patrullas y puestos automáticos; contratos con empresas privadas |
| 3 | Política general, presupuesto, guerra | Virreyes/consejos de sector |

- El jugador siempre puede **hacer zoom** y tomar control directo donde tenga presencia.
- **Costo de intervenir:** pasar por encima de un delegado puede dañar su lealtad o su autoridad.
- Los delegados son **notables** (06) con rasgos, competencia, ambiciones y lealtad propia.

---

## Progresión de las IA

- Las facciones IA pasan por las mismas fases con las mismas condiciones, **dentro de la banda de equilibrio** (arriba).
- Las civilizaciones pueden empezar en eras distintas (algunas ya interestelares, otras caídas); ver 14. Las que parten por delante del jugador no superan el techo de poder.
- La frontera puede estar ocupada cuando llegás si el ritmo del mundo está activo.
- Los **líderes** de cada civilización influyen en su ritmo y en su actitud hacia vos (10, 20).

## Ritmo orientativo (a validar con prototipos)

Sin duración fija por fase. Estimación de referencia, con ritmo del mundo "Activo":

| Fase | Horas de juego aproximadas |
|---|---|
| 1 (desde cero) | 20–50 (la ruta tecnológica del MVP equivale a ~25–30 años de juego, 22) |
| 2 | 20–60 |
| 3 | abierto |

## Conexiones

01 (niveles de detalle, ritmo del mundo) · 04 (relevamiento, logística) · 05 (apoyo público, colonias) · 06 (notables, líderes) · 07 (diseños de fábrica, programas) · 08 (idiomas, especies de inicio) · 09 (secesión, régimen, presión social) · 10 (primer contacto, coaliciones) · 13 (eras de tecnología) · 14 (inicios y generación) · 16 (relación visible, objetivos) · 20 (líderes y regímenes) · 22 (datos del MVP).

## Preguntas abiertas

- ¿Qué tan duro es el muro entre Fase 1 y 2? ¿Hay ayuda posible (comprar tecnología o pasaje a otra potencia, estilo "cliente")?
- Calibrar el ritmo real de cada fase y el costo de las eras de tecnología.
- Valores concretos del apoyo público y su relación con el gasto espacial (05, 09, 22).
- Cuándo y cómo se introducen las **regiones abstractas** (colonias grandes, después del MVP).
- Definir el **poder** comparable entre facciones para la banda de equilibrio.
