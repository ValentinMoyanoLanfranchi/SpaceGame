# 14 — Universo y generación procedural

> v0.3 · Agrega el **catálogo de inicios** y la **banda de equilibrio**. **El inicio por defecto y el MVP usan un sistema solar fijo y real (más 2 asteroides ficticios y un punto de salto oculto), no generado** (22); la generación procedural se usa para el resto del universo y para otros inicios. La Tierra es un único nodo en el MVP.

## Estructura jerárquica

```
Galaxia (región jugable, mapa 2D)
 └► Sectores (10–30 sistemas)
     └► Sistemas (estrella/s + cuerpos, mapa 2D cenital)
         └► Cuerpos celestes
             └► Regiones (abstractas, sólo en planetas y lunas grandes habitados; no en la Tierra del MVP)
```

La galaxia es un **grafo de sistemas unidos por puntos de salto** (04, 13). Cada sistema se muestra como un mapa 2D cenital con órbitas en rieles (circulares en el MVP, 21).

---

## Estrellas

- Distribución realista de tipos (la mayoría enanas rojas M; pocas tipo G).
- Sistemas binarios y múltiples.
- **Luminosidad** → zona habitable y energía solar.
- **Actividad** → frecuencia de fulguraciones (12).

## Cuerpos celestes

| Tipo | Características | Usos típicos |
|---|---|---|
| Rocosos (tipo Mercurio) | Sin atmósfera, calientes | Minería, energía solar |
| Terrestres / super-tierras | Variable; super-tierras con pozo gravitatorio profundo | Colonización, agricultura |
| Oceánicos | Superficie cubierta de agua | Volátiles |
| Helados | Agua congelada | Volátiles, combustible |
| Desérticos | Secos, tormentas | Minerales |
| Gigantes gaseosos | Hidrógeno, helio-3 | Extracción atmosférica, hábitats flotantes |
| Gigantes de hielo | Volátiles pesados | Combustible, química |
| Lunas | Enorme variedad; calentamiento por mareas | Bases, minería |
| Planetas enanos | Baja gravedad | Bases, minería |
| Cinturones de asteroides (C/S/M) | Carbonáceos, silicatos, metálicos | Minería sin pozo gravitatorio |
| Cometas | Volátiles en órbitas excéntricas | Agua y combustible |
| Planetas errantes | Sin estrella | Exóticos, escondites |
| Anomalías | — | Exploración, misterio |

Generación física: masa, radio y composición → gravedad, atmósfera, temperatura, recursos (yacimientos, 04). Cada cuerpo tiene además una **dificultad de detección** y un **estado de conocimiento** (02, 22).

---

## Catálogo de inicios

Cada inicio define especie, población, tecnología, conocimiento del sistema y estado del mundo (02). Estado de cada uno:

| Inicio | Sistema | Estado |
|---|---|---|
| **Tierra — Era pre-espacial (default)** | **Sistema solar real fijo** + 2 asteroides ficticios (Aurelio, Brisa) con cristales de presión + punto de salto oculto (22) | **Definido (MVP)** |
| Era espacial temprana | Sistema solar real, parcialmente relevado, con satélites y una estación | Propuesta |
| Mundo de otra especie | Sistema generado, con un planeta de origen adecuado a la especie (08) | Propuesta |
| Civilización caída | Sistema generado; tecnología avanzada y base industrial reducida (la premisa original de Diáspora y Colapso, 00) | Propuesta |
| Cliente / observador | Variantes de otros inicios con relaciones especiales (10) | Propuesta |

## Sistema y planeta inicial (inicio por defecto)

- **Sistema solar real:** 21 cuerpos con datos reales (planetas, lunas principales, asteroides, Sol), más 2 asteroides ficticios y un **punto de salto oculto** a 14 UA (zona gravimétrica 3, 22). No hay procedural en el MVP.
- **La Tierra es un único nodo, sin regiones ni bloques** (05): 2.500 M de habitantes, 4 estratos, 4 grupos de interés. Las regiones abstractas se usan en planetas y lunas grandes **a partir de la Fase 2 o después del MVP** (colonias grandes, mundos de otras especies).
- **Sin presencia espacial inicial:** no hay naves ni estaciones; todo se investiga y construye (02). Estado de conocimiento inicial: Tierra **Explorado**, Luna **Sospecha**, Sol **Observado**, el resto desconocido (22).
- **Sin vecinos ni amenazas armadas en el sistema local** durante la Fase 1: no hay piratas, rivales ni ruinas activas; las demás facciones están más allá del punto de salto (02).
- Para **otros inicios con sistema generado**, el sistema del jugador y su planeta capital se generan con mucho más detalle que el resto: el planeta capital puede tener regiones (propuesta: ~10–30, sin los 30–80 originales) y los demás cuerpos menos según su tamaño (1–10); puede haber **vecinos en el mismo sistema** según los parámetros del mundo.

## Red de puntos de salto

- Cada sistema tiene 1–5 puntos de salto (distribución a definir).
- **Algunos son ocultos** hasta que un relevamiento gravimétrico (por zonas, 04, 22) o un evento los revela.
- Algunos puntos pueden **abrirse** durante la partida (por civilizaciones o fenómenos): fuente de **anomalías** e incursiones (11, Fase 2).
- La red completa se genera al inicio de forma **gruesa**; los detalles de cada sistema se generan al **relevarlo** (semilla determinista por sistema, 21).

---

## Generación de historia (inspirado en Dwarf Fortress)

Antes de empezar, se corre la simulación en **modo rápido y grueso** durante N años (configurable, p. ej. 200–1.000) para generar el estado del **resto del universo** (lo que está más allá del sistema inicial en el inicio por defecto):
- Facciones nacen, guerrean, migran, colapsan.
- Se fundan y destruyen colonias y estaciones.
- Aparecen figuras históricas, artefactos, religiones, leyendas.
- **Piratas y bases** surgen de las zonas sin ley dejadas por el colapso (sólo en regiones ya fuera del sistema local en la Fase 1).

**Resultado:** el estado actual del mundo + un **archivo histórico** que el jugador descubre jugando (ruinas con historia real, reclamos territoriales antiguos, odios heredados).

### Premisa: La Diáspora y el Colapso (adaptada)
Capa más antigua: la civilización que colonizó la región y colapsó (00). Deja ruinas, tecnología, planetas sembrados de poblaciones aisladas, puntos de salto olvidados. **Con el inicio por defecto, la premisa describe el mundo que el jugador encuentra al salir de su sistema**; con el inicio "Civilización caída" conserva su forma original (00).

---

## Banda de equilibrio y escalado de las IA

Las civilizaciones generadas se **escalan respecto del jugador** (02):
- Poder de cada facción relevante limitado a `poder_jugador × (1 + banda)`, con `banda ∈ [−0,5; +0,5]` sorteada al crear la partida.
- Límite de sistemas proporcional al del jugador (más un margen inicial).
- Se puede desactivar ("sin límites").
- Las facciones lejanas que no pueden contactar al jugador evolucionan libremente hasta entrar en el radio de contacto.

## Contenido generado

**Puntos de interés:** ruinas, pecios, anomalías, bases abandonadas, nidos de fauna, tesoros, **cementerios de naves de batallas que realmente ocurrieron** en la historia generada, **bases piratas**, estaciones precursoras.

**Civilizaciones y especies:** una selección de las especies del núcleo (08) y generadas proceduralmente, algunas ya interestelares, otras caídas, otras con regímenes totalmente estatales. Cada una con su líder (20) y su disposición base (08).

## Parámetros de mundo (configurables)

Semilla · tamaño de la galaxia · densidad de civilizaciones · agresividad general · frecuencia de catástrofes · abundancia de recursos · duración de la historia previa · **inicio (escenario)** · era tecnológica inicial · presencia precursora · especies presentes · **ritmo del mundo (Activo / Lento / Relajado)** · **banda de equilibrio (rango y on/off)** · **densidad de piratería** · **incursiones por puntos de salto desconocidos (on/off e intensidad)** · **servidumbre habilitada** · **mercado privado habilitado**.

## Conexiones

01 (simulación previa, ritmo del mundo) · 02 (inicios, banda de equilibrio) · 04 (recursos, puntos de salto, relevamiento) · 05 (regiones) · 08 (especies) · 09 (facciones iniciales, piratas) · 11 (amenazas) · 12 (parámetros físicos) · 13 (ruinas, puntos de salto propios) · 15 (archivo histórico) · 20 (líderes) · 21 (semilla determinista) · 22 (datos del sistema solar del MVP).

## Preguntas abiertas

- Tamaño de la galaxia por defecto (¿100 sistemas? ¿500?).
- Cantidad de regiones por cuerpo en los inicios con sistema generado.
- Distribución de puntos de salto por sistema y proporción de ocultos.
- ¿Los sistemas lejanos se generan en detalle al relevarlos o antes? (Propuesta: al relevarlos, con semilla determinista.)
- Qué hay más allá del punto de salto tras el MVP (primer sistema generado, civilización contactable).
