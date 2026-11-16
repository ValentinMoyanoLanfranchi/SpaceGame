# 05 — Colonias y población

> v0.3 · **En el MVP la Tierra es un único nodo, sin regiones ni bloques** (decisión 60). Las regiones abstractas quedan reservadas para colonias grandes y para después del MVP. Datos iniciales en 22.

## Modelo: pops agregados + notables

- La población se modela como **pops**: grupos que comparten especie, cultura, clase, profesión y, cuando existen regiones, **región**. No hay individuos de a pie.
- Los **notables** (06) son los únicos individuos: gobernadores, líderes locales, directores, científicos, magnates.
- El jugador gestiona por **instalaciones, servicios y leyes**, no persona por persona.

### Escala de modelado

| Escala | Estructura | Cuándo |
|---|---|---|
| **Planeta capital (MVP: Tierra)** | **Un único nodo**: población agregada de 2.500 M, 4 estratos (62 / 30 / 5 / 3 %), 4 grupos de interés + población general | MVP |
| **Colonias pequeñas** (Luna, Marte, estaciones, asteroides) | Un nodo por asentamiento, con pops propios | MVP |
| **Planetas con regiones abstractas** | Cada planeta o luna grande se divide en regiones (sin mapa de superficie) con clima/bioma, recursos, población e infraestructura propios | Después del MVP; colonias grandes, otros mundos habitados, capitales de especies alienígenas |

> Motivo (decisión 60): dividir la Tierra en bloques regionales complicaba el modelo sin aportar a un juego que empieza en el espacio. Las tensiones internas se expresan con **grupos de interés y estratos**, no con geografía.

---

## Regiones (cuando se usan)

Atributos: bioma/entorno, habitabilidad por especie, yacimientos, infraestructura (energía, agua, puertos, transporte interno), población (pops), seguridad, malestar, propietario/controlador.

- Una colonia grande es un conjunto de regiones habitadas bajo un mismo gobierno local.
- Las regiones pueden pasar de dueño (guerra, secesión) y hacer que un planeta tenga múltiples facciones.
- Los eventos ambientales (12) afectan regiones concretas.

---

## Atributos de un pop

Tamaño · especie · cultura · religión/ideología · estrato social · profesión · nivel educativo · ingresos y ahorros · necesidades satisfechas · felicidad · **lealtad** (a la facción gobernante) · **radicalismo** · salud · distribución de edades.

---

## Apoyo público (Fase 1)

Indicador agregado del Estado sobre el programa espacial y el gobierno (fórmula y valores iniciales en 22). Depende de necesidades cubiertas, empleo, gasto social frente al gasto espacial, logros espaciales y fracasos (lanzamientos fallidos, muertes). Determina **la fracción del ingreso fiscal destinada al espacio** (`0,01 + 0,001 × apoyo` del ingreso anual, ≈ 600 M cr/año iniciales) y la intensidad de las presiones políticas (09).

## Grupos de interés (Fase 1)

La Tierra del MVP tiene **4 grupos de interés** más la población general (detalle en 22, régimen y rasgos en 09 y 20): presionan por prioridades distintas (gasto social, industria, ciencia, seguridad/nacionalismo) y condicionan el apoyo público y las leyes.

---

## Necesidades (pirámide)

| Nivel | Ejemplos | Si falla |
|---|---|---|
| 1. Supervivencia | Aire respirable, agua, comida, temperatura, presión, gravedad tolerable, protección contra radiación | Muerte, huida masiva |
| 2. Seguridad | Vivienda, orden público, salud, protección frente a piratas y ataques | Migración, crimen |
| 3. Pertenencia | Comunidad, familia, culto, respeto a su cultura | Aislamiento, radicalización |
| 4. Comodidad | Bienes de consumo, ocio, servicios | Descontento |
| 5. Estatus / lujo | Según clase | Descontento de élites (peligroso: tienen poder) |
| 6. Ideológicas | Libertades, leyes alineadas con su ideología | Protestas, apoyo a facciones rivales |

Las necesidades varían por **especie y cultura** (08). En el MVP, la Tierra consume una canasta agregada por estrato (18, 22).

---

## Estratos sociales (ejemplo)

Marginados · Trabajadores · Técnicos/artesanos · Administrativos/clase media · Intelectuales/clero · Militares · Élites (propietarios, aristocracia, ejecutivos y **empresarios**).

- **Movilidad** vía educación y riqueza.
- Restricciones legales según el régimen (castas, ciudadanía por especie, etc. — ver 09, 20).
- **Servidumbre / trabajo forzado** (opcional, ver 09, 19): pops sometidos con otros derechos, productividad y riesgo de rebelión.

---

## Demografía

- **Nacimientos:** tasa por especie × condiciones (comida, vivienda, seguridad, cultura).
- **Muertes:** edad, salud, catástrofes, guerra.
- **Migración:** factores de atracción/expulsión (salarios, seguridad, libertad, familiares). Refugiados de guerras y catástrofes ajenas.
- **Pirámide de edades:** una colonia vieja tiene poca fuerza laboral.
- **Generaciones nativas:** los nacidos en una colonia desarrollan identidad propia (alimenta la divergencia → secesión).
- **Colonias dependientes:** las colonias en cuerpos sin atmósfera/biosfera (Luna, Marte en el MVP) requieren infraestructura y suministro constante (agua, aire, energía, repuestos); sin ellos, mueren.

---

## Tipos de asentamiento

- **Por función:** puesto avanzado, colonia minera, agrícola, industrial, guarnición militar, científica, puerto/nodo logístico, capital.
- **Por entorno:** al aire libre, cúpula presurizada, subterráneo, hábitat orbital (rotatorio para gravedad), estación en asteroide, ciudad flotante (gigante gaseoso), submarina.

Cada entorno tiene costos, riesgos (brechas, fallas de soporte vital) y necesidades particulares.

## Instalaciones y servicios

- Las instalaciones funcionan como **capacidades** (minas, fábricas, refinerías, laboratorios, astilleros, puertos, sensores, defensas). Se construyen con recursos y producción (07). El MVP define 17 tipos de instalación (22).
- **Servicios:** salud · educación · seguridad/policía · emergencias · ocio · culto · administración · infraestructura. Cada uno con **cobertura** y **calidad**; consumen personal calificado y bienes. Los servicios pueden ser **estatales o privados** (03).

---

## Gobierno local

- **Carta de colonia** — nivel de autonomía:
  - *Directa:* el jugador controla todo.
  - *Administrada:* gobernador con directivas.
  - *Autónoma:* leyes locales propias, tributo fijo.
  - *Dominio/autogobierno:* casi independiente, aliada.
- **Gobernador:** notable (06) con rasgos y competencia (ficha en 20); sigue **directivas** (prioridades: crecimiento, producción, seguridad, lealtad; presupuesto; prohibiciones).
- **Corrupción** local según supervisión, distancia y carácter del gobernador.

---

## Malestar local (unrest)

### Fuentes
Necesidades insatisfechas · desigualdad · impuestos altos · represión · discriminación por especie/cultura/religión · desempleo · guerra impopular · derrotas militares · propaganda enemiga · líderes carismáticos · fervor religioso · escasez súbita · **inseguridad (piratas, incursiones, Fase 2)** · servidumbre · gasto espacial percibido como excesivo (Fase 1).

### Escalera de escalada

| Nivel | Estado | Efectos |
|---|---|---|
| 0 | Calma | — |
| 1 | Descontento | Quejas, productividad ↓ |
| 2 | Protestas | Manifestaciones, presión política |
| 3 | Huelgas / boicot | Producción se detiene en sectores |
| 4 | Disturbios / sabotaje | Daños, saqueos, víctimas |
| 5 | Insurgencia | Grupos armados, regiones "grises" fuera de control |
| 6 | Rebelión abierta / secesión | Nace una facción rebelde; si es grande → guerra civil (09) |

- **Histéresis:** es más fácil escalar que desescalar.
- Los notables locales (líderes sindicales, predicadores, caudillos) aceleran o frenan la escalada.
- **MVP:** la Tierra sin regiones sólo llega al nivel 3 (huelgas/boicot con efecto sobre producción y apoyo); insurgencia, secesión y guerra civil requieren regiones (después del MVP).

### Respuestas posibles
Concesiones y reformas, inversión en servicios, propaganda, negociación y cooptación, **represión** (eficaz a corto plazo, radicaliza a largo plazo), medidas extremas (se registran como atrocidades, con consecuencias diplomáticas e históricas).

---

## Identidad y divergencia

Las colonias lejanas **derivan culturalmente** con las generaciones (dialecto, valores, costumbres), lo que reduce la lealtad. Los lazos se mantienen con comercio, comunicaciones, educación compartida, migración y eventos comunes. (Efecto relevante desde la Fase 2; en el MVP las colonias son demasiado jóvenes.)

## Conexiones

02 (capacidad administrativa, Fase 1) · 03 (consumo, trabajo, servicios) · 04 (abastecimiento) · 06 (notables) · 08 (necesidades por especie/cultura) · 09 (leyes, grupos de interés, guerra civil, servidumbre) · 11 (seguridad, cansancio de guerra) · 12 (catástrofes) · 19 (leyes laborales) · 20 (gobernadores, regímenes) · 22 (datos de la Tierra y las colonias).

## Preguntas abiertas

- ¿Cuándo y cómo se introducen las regiones abstractas para planetas con más de una colonia grande (después del MVP)?
- ¿Simular familias dentro de pops?
- ¿Qué tan finamente se gestionan zonificación y servicios a escala de colonia?
- Calibrar el apoyo público y la fracción del presupuesto espacial (22).
