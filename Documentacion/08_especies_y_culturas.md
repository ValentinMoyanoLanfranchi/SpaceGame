# 08 — Especies, razas y culturas

> v0.3 · Agrega disposiciones base de cada especie en los 4 ejes de personalidad (20), factores de idioma para el primer contacto (02, 10) y la variante de régimen de colmena (20). **Humanos = especie por defecto del jugador (inicio en la Tierra);** las demás especies son jugables como otros inicios o aparecen como civilizaciones de la Fase 2.

## Tres capas distintas

| Capa | Qué es | Cambia… |
|---|---|---|
| **Especie** | Biología: necesidades, tolerancias, aptitudes | Muy lento (generaciones, ingeniería genética) |
| **Cultura** | Valores, costumbres, idioma aprendidos | Décadas (deriva, sincretismo) |
| **Facción** | Organización política | En cualquier momento |

Una especie tiene muchas culturas; una facción puede ser multiespecie y multicultural.

---

## Parámetros de especie

### Biología
- Bioquímica (carbono/agua, silicio, amoníaco…).
- Atmósfera respirable, rango de temperatura, presión, gravedad tolerada, tolerancia a radiación.
- Dieta y metabolismo.
- Longevidad, edad de madurez, tasa reproductiva.
- Enfermedades propias y compartidas.

### Psicología base
Sociabilidad · agresividad · aversión al riesgo · individualismo↔colectivismo · necesidad de espacio. Se traduce en la **disposición base** en los 4 ejes de personalidad (Agresividad, Apertura, Fiabilidad, Codicia; −3 a +3), que es el punto de partida de los líderes y notables de esa especie (06, 20).

### Aptitudes
Bonificaciones/penalidades a aptitudes y habilidades de los notables (06) y a la productividad de los pops por profesión.

### Estructura social innata
Pareja, clan, colmena (eusocial), solitaria. Determina qué **regímenes** son naturales para la especie (p. ej. una especie eusocial tiende a Estados totalmente estatales; 09, 20).

### Factor de idioma
Cada especie tiene un **factor de traducción** que escala el tiempo de entendimiento en el primer contacto (02, 10):

```
tiempo_traducción ≈ 2 años × factor_especie ÷ nivel_COM-1.2 × 1,5
```

(`nivel_COM-1.2` es el nivel de la tecnología de xenología/comunicación; valores a calibrar.)

---

## Especies del núcleo (propuesta: 3–4 bien hechas)

| Especie | Concepto | Rasgos clave | Disposición base (Ag / Ap / Fi / Co) | Factor idioma | Ganchos de juego |
|---|---|---|---|---|---|
| **Humanos** | Generalistas — **especie del jugador por defecto** | Adaptables, reproducción media, alta variabilidad cultural | 0 / 0 / 0 / +1 | 1,0 | Se fragmentan en subespecies con el tiempo (lunares, orbitales) |
| **Kethari** | Litófagos basados en silicio | Lentos, longevos (300+ años), toleran calor y radiación; se alimentan de minerales | −1 / −2 / +2 / +1 | 2,0 | Compiten por minerales; rencores de siglos; culturas de memoria larga; cuesta ganarse su confianza pero cumplen |
| **Ossari** | Eusociales | Castas biológicas; individualidad sólo en las reinas | +2 / −1 / +1 / 0 | 1,5 | **Estados totalmente estatales** (sin empresas privadas), comercio Estado–Estado; poca revuelta, crisis si muere una reina; guerra masiva |
| **Drel** | Nómadas del vacío | Nacidos en gravedad cero; flotas-ciudad; sufren en gravedad alta | +1 / +2 / −1 / +2 | 1,2 | No reclaman planetas; comerciantes y saqueadores; diplomacia distinta |

> Las disposiciones base y los factores de idioma son **valores propuestos** a validar en los prototipos de diplomacia (10).

### Régimen de colmena (Ossari)
Los Ossari usan una **variante de régimen "colmena"** (20): el liderazgo recae en la reina o en una cohorte de reinas; el peso del líder es máximo, la inercia alta y la sucesión se resuelve por **crianza de una nueva reina** (el pool de candidatos son las larvas reales) en lugar de herencia o elección. La muerte de la reina sin sucesora provoca crisis (09).

### Expansión / generación procedural (post-núcleo)
**Vael** (anfibios de mundos oceánicos), **Sintéticos** (IA/robots conscientes, dilema de derechos), **Mhuun** (simbiontes), más especies **generadas proceduralmente** a partir de plantillas (con disposición base y factor de idioma sorteados). **Precursores:** civilización desaparecida, no jugable (ruinas y misterio de fondo).

Además: **fauna y flora no sapientes** — ecosistemas propios de cada planeta (12).

---

## Culturas

### Ejes de valores
Tradición ↔ progreso · Colectivo ↔ individuo · Pacífico ↔ marcial · Abierto ↔ xenófobo · Espiritual ↔ materialista · Jerárquico ↔ igualitario.

### Costumbres
Dieta, ritos, arquitectura, vestimenta, festividades, tabúes, roles sociales, actitud hacia otras especies. Afectan consumo (03), malestar (05) y diplomacia (10).

### Dinámica
- **Deriva cultural:** colonias aisladas se diferencian (05).
- **Sincretismo:** culturas en contacto se mezclan.
- **Políticas:** asimilación forzada, integración, multiculturalismo, segregación — cada una con efectos en malestar, productividad y estabilidad.

---

## Religiones e ideologías

- Generadas con **doctrinas** que afectan leyes, comportamiento y preferencias de consumo.
- Cismas, herejías, reformas, misioneros, lugares sagrados (pueden estar en territorio ajeno → conflicto).
- Ideologías seculares (políticas, económicas) funcionan con el mismo sistema.

---

## Convivencia interespecies

- Hábitats mixtos requieren infraestructura para varios entornos (más cara).
- Tensiones: xenofobia, competencia por recursos o trabajo, malentendidos.
- Beneficios: aptitudes complementarias (Kethari en entornos radiactivos, Drel en logística espacial).

## Idiomas y comunicación

- Barrera idiomática en el primer contacto, con **tres fases**: sin lenguaje (gestos y señales), traducción parcial, lenguaje común (02, 10). El tiempo depende del **factor de idioma** de la especie y de la tecnología de xenología (COM-1.2, 13).
- Los malentendidos generan incidentes diplomáticos.

## Modificación y adaptación

- Ingeniería genética, cibernética, *uplift*.
- **Adaptación por generaciones:** humanos en una luna de baja gravedad se vuelven "lunares" con rasgos propios.
- Conflictos éticos y políticos (09).

## Especie y régimen del jugador

La **especie y la cultura** del Estado inicial cambian necesidades, aptitudes, disposición base y regímenes naturales. **Humanos es el default** (inicio en la Tierra antes de la era espacial, 02, 22); elegir otra especie es otro escenario de inicio y cambia mucho el arranque (qué comer, dónde vivir, cómo crecer, qué régimen es viable).

## Conexiones

02 (primer contacto, inicios) · 05 (necesidades y demografía) · 06 (aptitudes, personalidad) · 09 (ciudadanía, regímenes) · 10 (primer contacto, afinidad, idiomas) · 12 (habitabilidad) · 13 (xenología, COM-1.2) · 14 (dónde aparecen) · 20 (ejes de personalidad, régimen colmena).

## Preguntas abiertas

- Confirmar las 4 especies del núcleo y cuáles son jugables (el MVP sólo usa Humanos).
- Calibrar las disposiciones base y los factores de idioma con prototipos de diplomacia.
- Tono de los Mhuun (si entran): ¿horror o drama político?
- ¿Cuántas especies generadas proceduralmente se permiten?
