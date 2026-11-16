# 01 — Simulación del mundo vivo

> v0.3 · Agrega el **ritmo del mundo** configurable, el planificador alineado con 21 y las notas técnicas actualizadas (Rust, determinista, dirigido por datos).

## Objetivo

Que el universo funcione como un sistema autónomo: facciones que crecen y caen, mercados que suben y bajan, piratas, guerras y catástrofes, **todo ocurriendo con o sin el jugador**. El jugador es un actor más que, con el tiempo, puede volverse relevante.

---

## Principios

1. **Mismas reglas para todos.** Jugador e IA usan los mismos sistemas (economía, logística, combate, diplomacia). La dificultad no se implementa con trampas ocultas; si existen bonificaciones, límites o amenazas extra de dificultad (por ejemplo la **banda de equilibrio**, 02), son explícitos y visibles.
2. **Simulación siempre activa, con fidelidad variable.** Nada se "congela" fuera de la pantalla; se simula con menos detalle.
3. **Causalidad trazable.** Cada evento guarda referencias a sus causas, para poder explicarlo (ver 15 y 16).
4. **Conservación.** Lo que existe no aparece ni desaparece al cambiar de nivel de detalle: población, stocks, naves y dinero se conservan.
5. **Nada aparece de la nada.** Toda nave, flota o amenaza tiene origen (una base, un punto de salto, una facción). Lo que "aparece" es algo que el jugador no había detectado.

---

## Niveles de detalle (LOD de simulación)

| Nivel | Qué se simula | Cuándo | Frecuencia |
|---|---|---|---|
| **N1 — Colonia / región** | Pops agregados, edificios como "capacidades", stocks locales, mercado local, notables locales | Todas las colonias | Diaria (producción, consumo); mensual (demografía) |
| **N2 — Planeta** | Regiones, política interna, mercado planetario, eventos ambientales | Planetas con colonias | Diaria / semanal |
| **N3 — Sistema** | Cuerpos en rieles, flotas, tráfico comercial, sensores, patrullas, combate táctico | Sistemas con presencia relevante, contactos o conflicto | Desde 5 s hasta 30 días según el incremento |
| **N4 — Sector / galaxia** | Facciones lejanas, tráfico agregado, economía estadística, eventos de gran escala | Todo lo demás | Semanal / mensual |

> En el **MVP** (21) sólo hay un sistema, así que se simula en N3 y N1; los niveles N2 y N4 se agregan con la Fase 2.

### Promoción y degradación ("foco" de detalle)

- **Promoción:** cuando un sistema entra en foco (contacto hostil, conflicto, presencia del jugador, evento relevante), las flotas pasan de "nodos que se mueven por la red de rutas con una hora de llegada" a **objetos con posición y sensores**, y se simula con pasos finos.
- **Degradación:** al salir del foco, las flotas vuelven a moverse como nodos agregados; se preservan notables, naves con nombre/historia, daños y cambios estructurales (instalaciones construidas o destruidas).
- **Combate fuera de foco:** se resuelve con un modelo automático **calibrado contra el combate táctico** (mismas estadísticas). Si un enfrentamiento agregado da resultados distintos a lo que el táctico habría dado, se rompe la credibilidad → calibrar con corridas automáticas.

---

## Tiempo, incrementos y ticks

El jugador elige el incremento (5 s · 1 h · 1 día · 5 días · 30 días). Un planificador **multi-frecuencia** ejecuta cada sistema cuando corresponde:

| Frecuencia | Sistemas |
|---|---|
| Sub-pasos finos (segundos) | Movimiento y combate en sistemas en foco |
| Horaria | Reabastecimiento, tránsitos cortos, patrullas |
| Diaria | Producción, consumo, **investigación (acumulación de PI)**, logística, movimiento de flotas |
| Semanal | Mercados y precios, decisiones políticas y diplomáticas de la IA, malestar |
| Mensual | Demografía, empresas privadas, sociedad (apoyo público), presupuesto, balanza comercial |
| Anual | Resúmenes históricos, deriva cultural |

**Interrupciones:** cualquier evento marcado como relevante detiene el avance. Además, ante un contacto hostil el juego puede **ralentizarse automáticamente** y abrir una ventana de decisión (ver 11 y 16).

Cada planeta puede tener **calendario local** (día y año) además del **calendario estándar** común.

---

## Ritmo del mundo (configurable)

Las otras civilizaciones **siguen desarrollando su poder espacial** mientras el jugador avanza (02). Al crear la partida se elige:

| Modo | Efecto |
|---|---|
| **Activo** *(por defecto)* | Las IA avanzan a su ritmo; si el jugador se demora, otros toman los mejores sistemas |
| **Lento** | Las IA avanzan con ritmo reducido (≈ ×0,5) |
| **Relajado** | El avance de las IA se **desactiva** hasta el primer contacto |

La **banda de equilibrio** (02) limita cuánto más (o menos) fuertes pueden ser las IA respecto del jugador; también es configurable. Los dos ajustes son visibles en la configuración de la partida y se anotan en la crónica si se cambian.

---

## Agentes y toma de decisiones

### Facciones (IA en capas)

- **Estratégica:** objetivos de largo plazo (expandirse, sobrevivir, enriquecerse, imponer su ideología, venganza). Evaluación por **utilidad** + planes de varios pasos.
- **Operacional:** asignar flotas, rutas comerciales, proyectos de construcción e investigación, patrullas.
- **Táctica:** combate, escoltas, persecución.
- **Personalidad:** derivada de **especie, cultura, régimen y líder** (10, 20). Un cambio de líder puede cambiar el rumbo.

### Notables
Tienen **ambiciones** (riqueza, poder, venganza, conocimiento, fe, familia). Pueden cambiar de bando, fundar empresas o facciones, traicionar, morir. Un contrabandista puede terminar siendo señor de la guerra; un empresario, magnate de una megacorporación.

### Pops
Comportamiento estadístico: consumir, trabajar, migrar, protestar, radicalizarse.

### Empresas IA
Privadas: invierten donde hay margen, construyen instalaciones, despachan cargueros, desarrollan tecnología civil, quiebran. Estatales (de facciones IA): siguen el plan del Estado.

### Piratas y saqueadores
Grupos con **bases reales** que atacan objetivos **rentables y débiles** (zonas de baja seguridad), sean o no del jugador. Aparecen desde la Fase 2 (en la Fase 1 no hay amenazas de naves en el sistema local, 02).

---

## El jugador no es especial

- Ninguna facción tiene comportamiento "anti-jugador" programado. Reaccionan con las mismas reglas que aplican a otras facciones.
- **Relevancia:** índice calculado por cada facción respecto de cada otra (poder militar, económico, cercanía, reputación, interdependencia). Con relevancia baja, el jugador es ignorado.
- Amenazas lejanas (incursiones por puntos de salto desconocidos) existen como **dificultad opcional**, siempre con aviso previo (ver 11, 14).

---

## Director / narrador (opcional)

| Modo | Comportamiento |
|---|---|
| **Indiferente** (por defecto) | Sin director. Todo emerge de la simulación. |
| **Cronista** | Modula tasas *globales* de eventos aleatorios (catástrofes, anomalías, migraciones) para evitar períodos muy largos sin nada. |
| **Sereno / Implacable** | Variantes con más o menos "temperatura". |

**Regla de oro:** el director **nunca elige objetivos**, sólo ajusta tasas globales. El jugador nunca es blanco intencional.

---

## Información y niebla (decidido)

La **información es un recurso**. El jugador sabe lo que ven sus sensores, patrullas, puestos de vigilancia y red de inteligencia, lo que reportan embajadas y lo que traen comerciantes y rumores.

- **Conocimiento del sistema:** cada cuerpo pasa por **estados de conocimiento** (Desconocido → Sospecha → Observado → Relevado → Explorado) que sólo mejoran con acciones del jugador (02, 04). Los telescopios dan pistas con rangos amplios.
- **Dentro de un sistema:** información casi inmediata si hay sensores; el jugador **sabe más que en Aurora**: alertas de contacto con tiempo estimado de llegada, procedencia probable y composición estimada.
- **Entre sistemas: latencia mitigable con relés.** La información viaja con las naves o por **relés de comunicaciones** (COM-2.1) y sensores en puntos de salto, que la reducen. El mapa muestra "último dato: hace 43 días". Esto da valor a la infraestructura de comunicaciones y a los puestos de vigilancia (04, 11, 13).
- **Líderes ajenos:** su ficha se descubre por niveles según la inteligencia (20).
- **Capacidad administrativa:** la latencia entra en el costo de gobernar territorios lejanos (02).
- **Precisión:** estimaciones con márgenes ("flota enemiga: 8–14 naves").
- **Desinformación:** el espionaje puede plantar datos falsos; la propaganda distorsiona noticias.

---

## Persistencia y determinismo

- Semilla de mundo + semillas por sistema/región.
- Registro de eventos persistente (alimenta 15).
- **Simulación determinista** (misma semilla y mismos comandos → mismo resultado): RNG con semilla, orden estable de iteración, sin tiempo real en la lógica (21).

## Notas técnicas (ver 21 para el detalle vinculante)

- **Núcleo en Rust**, independiente del motor, con **Godot 4 + gdext** para la interfaz.
- Estructuras simples con IDs tipados y arenas (se puede migrar a ECS si hace falta).
- La acción del jugador entra por **comandos**; la UI lee instantáneas.
- Paralelismo por sistema y nivel; N2–N4 en hilos de fondo (con reducción determinista).
- Metas orientativas a largo plazo: cientos de sistemas, miles de notables, cientos de millones de habitantes en pops, decenas de miles de naves y empresas simuladas.
- Datos del juego (especies, bienes, recetas, tecnologías, diseños) en **archivos JSON** → moddeable (22).

---

## Conexiones

02 (fases, ritmo del mundo, banda de equilibrio) · 03/04 (economía y logística en N1–N4) · 09/10 (IA de facciones y líderes) · 11 (foco de combate) · 15 (registro causal) · 16 (visualización de información incierta) · 20 (conocimiento de líderes) · 21 (arquitectura).

## Preguntas abiertas

- ¿Cuántos sistemas pueden estar en foco a la vez?
- ¿Modo "Indiferente" por defecto con períodos aburridos, o "Cronista"?
- ¿Cuánta información "omnisciente" se da en dificultad baja?
- Alcance y costo de los relés de comunicaciones.
- Cadencias definitivas de cada subsistema tras los primeros prototipos (21).
