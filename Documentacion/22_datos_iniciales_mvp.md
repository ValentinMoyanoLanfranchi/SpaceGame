# 22 — Datos iniciales del MVP

> v0.1 · Valores concretos para implementar la Fase 1 del MVP (21): sistema solar, bienes, instalaciones, tecnologías, diseños de fábrica, Tierra y sociedad, empresas privadas y formato de datos. **Todos los números son iniciales y están pensados para calibrarse** con el prototipo (CLI sin interfaz, 21). Se basan en los docs 02, 04, 07, 13 y 18.

## 0. Convenciones

- **Tiempo:** año de 365 días (parámetro `days_per_year`, máximo 65.535; lo usan el calendario y la cadencia anual); calendario simple; fecha de inicio configurable (por defecto "Año 1, día 1").
- **Cadencias del planificador:** semana = 7 días, mes = 30 días, año = 365 días (el mes no encaja en el año: 12 meses = 360 días). Las fronteras se cuentan desde el inicio de la partida, no desde la fecha de inicio del calendario: en 365 días corren 365 veces un sistema diario, 52 uno semanal, 12 uno mensual y 1 uno anual. Un incremento largo ejecuta un sistema una vez por cada frontera que cruza (D30 = 30 corridas diarias y 1 mensual). Orden de ejecución estable: instante, prioridad fija y luego id.
- **Dinero:** créditos (cr). "M cr" = millones. Los precios por tonelada son los de 18.
- **Masa:** toneladas (t); reservas en megatoneladas (Mt).
- **Distancias:** UA en órbitas heliocéntricas; km en lunas.
- **Órbitas del MVP:** **circulares y coplanares** (e = 0), en rieles (04). Los elementos reales se aproximan; la fase inicial de cada cuerpo se sortea con la semilla.
- **IDs:** texto estable (`body.luna`, `good.hierro`, `tech.pro_0_1`, `design.lv_1`); los bienes llevan además su número de 18.

---

## 1. Sistema del MVP

**Sistema Solar real + elementos ficticios.** Planetas y lunas con datos aproximados reales, más **dos asteroides ficticios** ricos en un exótico y **un punto de salto oculto**.

### 1.1 Cuerpos (21)

`Dif.` = dificultad de detección por telescopio (0–10; ver 1.3). `Hab.` = habitabilidad para colonias (0–1). `Masa (kg)` = masa real aproximada (`mass_kg` en los datos), en notación científica (`7,35e22` = 7,35 × 10²²); la usa la vista con rangos (1.3). Las masas de Aurelio y Brisa son ficticias: salen de su radio con una densidad de ~3.500 kg/m³ (metálico) y ~1.500 kg/m³ (helado).

| ID | Cuerpo | Tipo | Padre | Órbita | Período (d) | Radio (km) | Masa (kg) | Dif. | Hab. |
|---|---|---|---|---|---|---|---|---|---|
| sol | Sol | Estrella | — | — | — | 695.700 | 1,989e30 | 0 | — |
| mercurio | Mercurio | Planeta | Sol | 0,387 UA | 88 | 2.440 | 3,30e23 | 3 | 0,05 |
| venus | Venus | Planeta | Sol | 0,723 UA | 225 | 6.052 | 4,87e24 | 2 | 0,02 |
| tierra | **Tierra** | Planeta | Sol | 1,000 UA | 365 | 6.371 | 5,97e24 | 0 | 1,0 |
| luna | Luna | Luna | Tierra | 384.400 km | 27,3 | 1.737 | 7,35e22 | 0 | 0,30 |
| marte | Marte | Planeta | Sol | 1,524 UA | 687 | 3.390 | 6,42e23 | 2 | 0,45 |
| fobos | Fobos | Luna | Marte | 9.376 km | 0,32 | 11 | 1,07e16 | 6 | 0,10 |
| deimos | Deimos | Luna | Marte | 23.463 km | 1,26 | 6 | 1,48e15 | 6 | 0,10 |
| aurelio | **Aurelio** *(ficticio)* | Asteroide | Sol | 2,20 UA | 1.200 | 40 | 9,4e17 | 7 | 0,10 |
| vesta | Vesta | Asteroide | Sol | 2,36 UA | 1.325 | 263 | 2,59e20 | 5 | 0,15 |
| ceres | Ceres | Planeta enano | Sol | 2,77 UA | 1.681 | 473 | 9,38e20 | 5 | 0,20 |
| palas | Palas | Asteroide | Sol | 2,77 UA | 1.686 | 256 | 2,04e20 | 6 | 0,10 |
| brisa | **Brisa** *(ficticio)* | Asteroide | Sol | 2,95 UA | 1.850 | 30 | 1,7e17 | 7 | 0,10 |
| higiea | Higiea | Asteroide | Sol | 3,14 UA | 2.030 | 217 | 8,7e19 | 6 | 0,10 |
| jupiter | Júpiter | Gigante gaseoso | Sol | 5,203 UA | 4.333 | 69.911 | 1,898e27 | 2 | — |
| io | Ío | Luna | Júpiter | 421.700 km | 1,77 | 1.822 | 8,93e22 | 4 | 0,02 |
| europa | Europa | Luna | Júpiter | 671.034 km | 3,55 | 1.561 | 4,80e22 | 4 | 0,10 |
| ganimedes | Ganímedes | Luna | Júpiter | 1.070.412 km | 7,15 | 2.634 | 1,48e23 | 4 | 0,15 |
| calisto | Calisto | Luna | Júpiter | 1.882.709 km | 16,7 | 2.410 | 1,08e23 | 4 | 0,15 |
| saturno | Saturno | Gigante gaseoso | Sol | 9,537 UA | 10.759 | 58.232 | 5,68e26 | 3 | — |
| titan | Titán | Luna | Saturno | 1.221.870 km | 15,9 | 2.575 | 1,35e23 | 5 | 0,20 |

### 1.2 Punto de salto

| ID | Tipo | Posición | Estado inicial | Conexión |
|---|---|---|---|---|
| `jp.alfa` | Punto de salto | **14,0 UA** del Sol, ángulo fijo 217° (estático respecto de la estrella) | **Oculto**: sólo lo revela el relevamiento gravimétrico (SEN-3.1) | Sistema destino fuera del MVP (cruzarlo es el cierre de la Fase 1) |

**Relevamiento gravimétrico por zonas:** el espacio entre 0 y 30 UA se divide en **6 zonas** de 5 UA de ancho. Una nave de exploración tarda **6 meses por zona** con un gravímetro (el tiempo se divide entre las naves asignadas). El punto de salto está en la **zona 3 (10–15 UA)**. Una vez hallado pasa a "Observado"; hay que **estudiarlo** (SEN-3.2) antes de cruzar.

### 1.3 Conocimiento: reglas de revelado

Estado inicial: **Tierra: Explorado · Luna: Sospecha · Sol: Observado · el resto: Desconocido.**

**Telescopios (poder):**

| Tecnología | Poder del telescopio |
|---|---|
| SEN-0.1 (inicial) | 3 |
| SEN-0.2 | 4 |
| SEN-1.1 | 7 |
| SEN-1.3 | 8 |

Un cuerpo pasa a **Sospecha** cuando `poder ≥ dificultad`, tras `meses = 2 + 2 × dificultad` de observación continua (con un observatorio activo). Con el equipo inicial se sospechan Venus y Marte y Júpiter (6 meses), Mercurio y Saturno (8 meses); las lunas de Júpiter necesitan SEN-0.2; asteroides, Fobos, Deimos y Titán necesitan SEN-1.1.

| Paso | Condición |
|---|---|
| Desconocido → **Sospecha** | Telescopio (arriba); datos con rangos amplios (masa ±50%, órbita ±10%, composición desconocida) |
| Sospecha → **Observado** | SEN-1.1 con cuerpo de dificultad ≤ 6, o sonda de sobrevuelo (SND-1); órbita y masa a ±5% |
| Observado → **Relevado** | Sonda orbital (SND-2) o nave REL-1 en órbita durante 3–6 meses; yacimientos con datos precisos |
| Relevado → **Explorado** | Aterrizaje, base o colonia |

Los **recursos** de un cuerpo sólo se conocen con precisión en estado Relevado; en Sospecha y Observado se muestran **rangos** (p. ej. "hierro: 1–10 Gt, accesibilidad media").

### 1.4 Yacimientos (reservas explotables, Mt) y accesibilidad (0–1)

Las reservas son lo **explotable** (no el total del cuerpo). La accesibilidad multiplica la producción de las minas.

| Cuerpo | Recursos (reserva Mt · accesibilidad) |
|---|---|
| **Tierra** | Mercado terrestre (ver 4.4); no se minan por programas |
| **Luna** | Silicatos 1.000.000 · 0,9 / Hierro 200.000 · 0,7 / Hielo (polos) 600 · 0,3 |
| **Marte** | Silicatos 2.000.000 · 0,9 / Hierro 500.000 · 0,7 / Hielo 5.000.000 · 0,5 / Cobre 5.000 · 0,3 |
| **Mercurio** | Hierro 1.000.000 · 0,2 / Silicatos 800.000 · 0,2 |
| **Fobos / Deimos** | Silicatos 20.000 · 0,8 / Hielo 500 · 0,6 |
| **Vesta** | Hierro 30.000 · 0,8 / Silicatos 80.000 · 0,9 / Cobre 500 · 0,7 |
| **Ceres** | Hielo 50.000 · 0,6 / Silicatos 100.000 · 0,8 / Hierro 10.000 · 0,7 |
| **Palas** | Silicatos 60.000 · 0,8 / Hierro 8.000 · 0,7 |
| **Higiea** | Hielo 20.000 · 0,6 / Silicatos 40.000 · 0,8 |
| **Aurelio** *(ficticio)* | Hierro 5.000 · 0,9 / Cobre 1.500 · 0,8 / **Cristales de presión 20 · 0,6** |
| **Brisa** *(ficticio)* | Hielo 8.000 · 0,7 / Silicatos 3.000 · 0,8 / **Cristales de presión 15 · 0,5** |
| **Júpiter, Saturno** | Hidrógeno (atmósfera, no agotable); requiere tecnología fuera del MVP (MIN-3.2) |
| **Europa, Ganímedes, Calisto, Titán, Ío, Venus** | Detallados en el MVP+ / contenido posterior (sólo observables) |

### 1.5 Colonias del MVP

Sólo se pueden fundar colonias **en la Luna y en Marte** (y bases/estaciones en órbita y asteroides). Requisitos: `VID-2.1` (hábitats superficiales) para superficie; `VID-1.2` para estaciones. Detalle en la sección 8.

---

## 2. Bienes del MVP (16)

Subconjunto de 18, con sus números. Las recetas están **simplificadas** respecto de 18 (sólo con bienes del conjunto).

| ID texto | # en 18 | Bien | Precio base (cr/t) | Perece | Legal |
|---|---|---|---|---|---|
| `good.hierro` | 1 | Mineral de hierro | 20 | — | L |
| `good.cobre_crudo` | 3 | Mineral de cobre | 60 | — | L |
| `good.silicatos` | 9 | Silicatos | 10 | — | L |
| `good.hielo` | 11 | Hielo de agua | 5 | — | L |
| `good.hidrogeno` | 12 | Hidrógeno | 40 | — | L |
| `good.biomasa` | 15 | Biomasa | 15 | 20 d | L |
| `good.cristales` | 16 | Cristales de presión *(ficticio)* | 3.000 | — | R |
| `good.acero` | 19 | Acero | 80 | — | L |
| `good.cobre` | 21 | Cobre refinado | 150 | — | L |
| `good.silicio` | 23 | Silicio de alta pureza | 200 | — | L |
| `good.agua` | 26 | Agua tratada | 15 | — | L |
| `good.combustible` | 27 | Combustible químico | 100 | — | L |
| `good.piezas` | 31 | Piezas mecánicas | 250 | — | L |
| `good.electronica` | 32 | Electrónica | 600 | — | L |
| `good.repuestos` | 37 | Repuestos y suministros | 300 | — | L |
| `good.alimentos` | 39 | Alimentos básicos | 40 | 30 d | L |

### 2.1 Recetas (MVP)

| Receta | Insumos | Producto | Instalación | Energía (MWh/t) |
|---|---|---|---|---|
| Acero | 2,0 t hierro | 1 t acero | Acería | 2 |
| Cobre refinado | 3,0 t cobre crudo | 1 t cobre | Refinería | 3 |
| Silicio | 3,0 t silicatos | 1 t silicio | Planta de silicio | 5 |
| Agua tratada | 1,2 t hielo | 1 t agua | Planta de agua | 1 |
| Hidrógeno (electrólisis, simplificada) | 3,0 t hielo | 1 t hidrógeno | Planta de electrólisis | 12 |
| Combustible químico | 0,8 t hidrógeno + 0,4 t agua | 1 t combustible | Planta de combustible | 6 |
| Piezas mecánicas | 0,8 t acero | 1 t piezas | Taller de precisión | 4 |
| Electrónica | 0,4 t cobre + 0,4 t silicio | 1 t electrónica | Fábrica de electrónica | 10 |
| Repuestos | 0,5 t piezas + 0,5 t electrónica | 1 t repuestos | Fábrica de repuestos | 6 |
| Alimentos básicos | 1,5 t biomasa | 1 t alimentos | Procesadora | 1 |
| Biomasa (granja cerrada) | 0,3 t agua + energía | 1 t biomasa | Granja cerrada | 15 |

*(Margen de las recetas es alto a propósito: incluye mano de obra, ingeniería y riesgo no modelados en el MVP; ver calibración.)*

### 2.2 Mercado

- **Por nodo** (planeta/luna/estación): `precio = precio_base × clamp((stock_objetivo / stock)^elasticidad, 0,25, 6,0)`, suavizado ±10%/día (18).
- **Cobertura y elasticidad:** alimentos y agua 30 d / 1,2; combustible 45 d / 1,0; repuestos 60 d / 0,8; minerales y refinados 60 d / 0,7; componentes 60 d / 0,6.
- **Mantenimiento:** toda instalación y nave consume repuestos: ~**2%/año** del valor de la instalación (18).

---

## 3. Instalaciones del MVP

| Instalación | Capacidad | Energía (MW) | Costo (M cr) | Tecnología | Dónde |
|---|---|---|---|---|---|
| Laboratorio general | 10 PI/mes | 1 | 60 | Inicial | Tierra, estaciones, bases |
| Observatorio terrestre | Poder de telescopio (ver 1.3) | 0,5 | 40 | Inicial | Tierra |
| Plataforma de lanzamiento | 1 lanzamiento/mes | 5 | 200 | PRO-0.1 | Tierra |
| Paneles solares | 10 MW a 1 UA (escala 1/d²) | — | 20 | ENE-0.1 | Cualquiera |
| Reactor de fisión | 50 MW | — | 120 | ENE-1.1 | Estaciones, bases |
| Mina robótica | 500 t/mes × accesibilidad | 3 | 40 | MIN-2.1 (superficie) / MIN-2.2 (asteroides) | Cuerpos |
| Acería | 200 t/mes | 8 | 50 | Inicial (Tierra) / MIN-2.3 (fuera) | — |
| Refinería de cobre | 100 t/mes | 6 | 40 | ídem | — |
| Planta de silicio | 100 t/mes | 10 | 60 | ídem | — |
| Planta de agua | 400 t/mes | 2 | 25 | MIN-2.3 | Fuera de la Tierra |
| Planta de electrólisis | 50 t/mes | 12 | 55 | MIN-2.3 | Fuera de la Tierra |
| Planta de combustible | 100 t/mes | 6 | 45 | MIN-2.3 | Fuera de la Tierra |
| Taller de precisión | 80 t/mes | 5 | 45 | MAT-1.2 | Fuera de la Tierra |
| Fábrica de electrónica | 30 t/mes | 8 | 70 | MAT-1.2 | Fuera de la Tierra |
| Fábrica de repuestos | 30 t/mes | 4 | 50 | MAT-1.2 | Fuera de la Tierra |
| Granja cerrada | 40 t/mes de biomasa | 2 | 35 | VID-2.3 (opcional) | Colonias |
| **Taller orbital (astillero)** | Construye diseños hasta 300 t; **10 t/mes** por dársena | 15 | 500 | MAT-1.2 | Órbita |

- En la **Tierra** las instalaciones industriales ya existen y alimentan el **mercado terrestre**; el jugador sólo construye lo espacial.
- **Mano de obra:** en el MVP se ignora (instalaciones automatizadas o con personal abstracto), salvo las colonias (sección 8).
- **Energía:** cada nodo tiene un balance de MW; si falta energía, la producción baja proporcionalmente.

---

## 4. Tierra y sociedad

### 4.1 Nodo Tierra (sin bloques regionales en el MVP)

Decidido: la Tierra es **un único nodo**, sin regiones ni bloques (sólo agregados), para no complicar el MVP. Las regiones abstractas (05) se reservan para colonias y para después.

| Parámetro | Valor inicial |
|---|---|
| Población | **2.500 millones** |
| Estratos (para demanda) | Trabajadores 62% · Clase media 30% · Élites 5% · Subsistencia 3% |
| Recaudación fiscal | **10.000 M cr/año** |
| Fracción máxima destinable al programa espacial | `0,01 + 0,001 × apoyo_público` (con apoyo 50 → 6% ≈ 600 M cr/año) |
| Gasto social obligatorio | ~60% de la recaudación |
| Crecimiento económico base | +1,5%/año (varía con inversión y apoyo) |

### 4.2 Grupos de interés (4)

| Grupo | Influencia | Aprobación inicial | Qué quiere |
|---|---|---|---|
| Industriales y comerciantes | 28 | 55 | Contratos, impuestos bajos, acceso a recursos |
| Trabajadores y sindicatos | 30 | 50 | Gasto social, empleo, salarios |
| Científicos y universidades | 12 | 70 | Presupuesto de investigación |
| Nacionalistas / seguridad | 15 | 45 | Prestigio, defensa, control de recursos |
| *(sin grupo / población general)* | 15 | 55 | Estabilidad y costo de vida |

```
apoyo_público = Σ (influencia_g × aprobación_g) / Σ influencia_g       # inicial ≈ 52
```

- Subir el gasto espacial **por encima de la fracción sostenible** reduce la aprobación de trabajadores; los éxitos (hitos) la suben.
- Eventos sociales simples (huelgas, protestas, escándalos) mueven aprobaciones y generan alertas (09).

### 4.3 Estado inicial del jugador

| Elemento | Valor |
|---|---|
| Tecnologías | SEN-0.1, COM-0.2, MAT-0.1 |
| Laboratorios | 10 (generales) |
| Científicos | 15 (habilidad de investigación 2–5; ≈ 120 PI/mes) |
| Instalaciones | 1 observatorio terrestre; 1 plataforma de lanzamiento *(requiere PRO-0.1 para usarla)* |
| Capital inicial | 400 M cr (reservas) + el presupuesto anual |
| Flotas | Ninguna |

### 4.4 Mercado terrestre

- El Estado y las empresas **compran en la Tierra** los 15 bienes (todos menos `good.cristales`) a `precio_base × 1,15`, hasta **2.000 t/mes por bien** (crece con la economía).
- La Tierra **compra** bienes del espacio a `precio_base × 0,7`, con demanda decreciente (hasta 500 t/mes por bien al inicio); permite exportar metales y combustibles y es la base de la rentabilidad privada.
- **Costo de lanzamiento** (ver 5): lo que domina el precio de todo lo que se sube a órbita (pozo gravitatorio, 02).

### 4.5 Costos de laboratorio y científicos
Cada laboratorio cuesta ~**15 M cr/año** de mantenimiento (incluye personal). Nuevos laboratorios: 60 M cr y científicos formados por ADM-0.2.

---

## 5. Diseños de fábrica del MVP (15)

Subóptimos a propósito (~70–80%, 07). Costos "en la Tierra" incluyen ingeniería y fabricación; los lanzadores son **desechables**.

| ID | Diseño | Tecnología | Masa (t) | Costo (M cr) | Capacidad / uso |
|---|---|---|---|---|---|
| `design.lv_1` | **LV-1** Lanzador ligero | PRO-0.1 | 120 | **30** por lanzamiento | 2 t a órbita baja |
| `design.lv_2` | **LV-2** Lanzador pesado | PRO-1.2 | 450 | **70** por lanzamiento | 15 t a órbita baja |
| `design.sat_o1` | SAT-O1 Satélite de observación | PRO-0.1, SEN-0.1 | 1,5 | 120 | Mejora de datos (poder +1) |
| `design.sat_c1` | SAT-C1 Satélite de comunicaciones | COM-0.1 | 2,5 | 150 | Cobertura; base de la red |
| `design.cap_1` | CAP-1 Cápsula tripulada | VID-0.1 | 6 | 400 | 3 tripulantes |
| `design.snd_1` | SND-1 Sonda de sobrevuelo | PRO-1.1, SEN-1.2 | 0,8 | 90 | Observa un cuerpo (→ Observado) |
| `design.snd_2` | SND-2 Sonda orbital | SEN-1.2, MIN-1.1 | 1,5 | 200 | Relevamiento prolongado (→ Relevado) |
| `design.est_1` | EST-1 Estación básica | VID-1.2 | 45 | 900 | 6 tripulantes, 3 módulos; laboratorios |
| `design.trn_1` | TRN-1 Transporte orbital | PRO-1.2 | 80 (seca) | 600 | 200 t de carga |
| `design.min_1` | MIN-1 Nave minera | MIN-2.2, PRO-1.2 | 60 (seca) | 500 | 100 t por viaje |
| `design.rel_1` | REL-1 Nave de relevamiento | SEN-2.1, PRO-2.2 | 40 | 450 | Relevamiento (→ Relevado) |
| `design.bas_1` | BAS-1 Base superficial | VID-2.1 | 120 | 1.500 | ~200 habitantes |
| `design.col_1` | COL-1 Colonizadora | VID-2.1 | 150 | 1.200 | ~500 colonos |
| `design.exp_1` | EXP-1 Exploradora de largo alcance | PRO-2.1, SEN-2.2 | 90 | 1.500 | Autonomía larga, gravímetro (con SEN-3.1) |
| `design.exp_2` | **EXP-2 Nave de salto** | PRO-3.2 | 150 | 4.000 | Cruza puntos de salto; **requiere 5 t de cristales de presión** |

- Los **gravímetros** (SEN-3.1) requieren **2 t de cristales de presión** por unidad; EXP-1 lleva un gravímetro.
- Los diseños con **materiales** (en vez de costo directo) se pueden construir en el **Taller orbital** con insumos locales (acero, piezas, electrónica) para evitar lanzar toda la masa: esa es la ventaja de MAT-1.2 y MIN-2.3.
- Lo que se sube a órbita se paga por el lanzamiento: `costo_subida = ceil(masa / capacidad_lanzador) × costo_lanzamiento`.
- **Reusables (PRO-0.2, fuera del MVP mínimo):** −30% del costo de lanzamiento.

---

## 6. Tecnologías del MVP (38 + 3 iniciales)

Las 38 tecnologías de la ruta que llega al **cruce del punto de salto** (el cierre del MVP). Costos y prerrequisitos en 13. Con SEN-0.2, SEN-2.2, MAT-2.2, ADM-2.2, PRO-2.1, ENE-2.1, SEN-3.2 y PRO-3.2 incluidas por ser prerrequisitos, el total sube a **38** (la ruta crítica de 17 tenía 30).

| Campo | Tecnologías (38) | PI aprox. |
|---|---|---|
| PRO (6) | 0.1, 1.1, 1.2, 2.1, 2.2, 3.2 | 62.300 |
| ENE (4) | 0.1, 1.1, 1.2, 2.1 | 13.800 |
| MAT (4) | 0.2, 1.1, 1.2, 2.2 | 13.000 |
| SEN (7) | 0.2, 1.1, 1.2, 2.1, 2.2, 3.1, 3.2 | 69.700 |
| COM (4) | 0.1, 1.1, 1.2, 2.1 | 11.100 |
| VID (4) | 0.1, 1.1, 1.2, 2.1 | 15.500 |
| MIN (4) | 1.1, 2.1, 2.2, 2.3 | 25.800 |
| ADM (5) | 0.1, 1.1, 2.1, 2.2, 2.3 | 17.900 |
| **Total** | **38** | **≈ 229.000** |

- **Iniciales (✔):** SEN-0.1, COM-0.2, MAT-0.1.
- A ~120 PI/mes inicial y creciendo a ~1.500 PI/mes, la ruta completa lleva **≈ 25–30 años de juego** (a calibrar).
- **Fuera del MVP** (quedan en 13): PRO-0.2, PRO-3.1, ENE-3.x, MAT-2.1, MAT-3.x, SEN-1.3, COM-2.2, COM-3.1, VID-2.2, VID-2.3, VID-3.1, MIN-2.4, MIN-3.x, ADM-0.2, ADM-3.x. (Nota: sin ADM-0.2 no hay más científicos formados; en el MVP se asume un flujo constante de graduados.)

### 6.1 Formato de datos (ejemplo)

```json
{
  "id": "tech.pro_1_1",
  "field": "PRO",
  "era": 1,
  "name_key": "tech.pro_1_1.name",
  "cost_pi": 2500,
  "prereqs": ["tech.pro_0_1", "tech.ene_0_1"],
  "unlocks": { "designs": ["design.snd_1"], "effects": [{ "type": "isp_multiplier", "value": 5 }] },
  "initial": false
}
```

---

## 7. Empresas privadas del MVP (3–4)

Aparecen cuando se cumple su condición y hay **retorno esperado ≥ 8%** (03, 19).

| Empresa (ficticia) | Aparece con | Rubro | Capital inicial (M cr) | Comportamiento |
|---|---|---|---|---|
| **Lanzadera Orbital S.A.** | ADM-1.1 | Servicios de lanzamiento (LV-1, LV-2) | 300 | Ofrece lanzamientos por t a órbita: `precio ≈ costo × 1,2`; baja con la competencia |
| **Transportes Orbitales** | ADM-1.1 + PRO-1.2 | Fletes entre cuerpos | 250 | Construye TRN-1 y fleta carga; viaja si el margen ≥ 5–10% |
| **Minera del Cinturón** | ADM-2.2 + MIN-2.2 | Minería de asteroides y Luna | 400 | Requiere **licencia/regalía** (10%); vende a la Tierra o al Estado |
| **Industrias Orbitales** *(opcional)* | MAT-1.2 + ADM-2.2 | Plantas fuera de la Tierra (acero, electrónica) | 350 | Invierte si hay demanda y energía |

- Rubros **reservados al Estado** por defecto: ninguno en el MVP (el jugador puede reservar).
- Pueden **quebrar** si el margen es negativo durante 12 meses.
- Impuestos en el MVP: **renta global** y **regalías de extracción** (el resto se difiere).

---

## 8. Colonias (Luna y Marte)

| Parámetro | Valor |
|---|---|
| Tamaño máximo inicial | Luna 5.000 hab · Marte 20.000 hab (limitado por hábitats y suministro) |
| Alojamiento | BAS-1: 200 hab por base |
| Consumo por 1.000 habitantes/mes | Alimentos 15 t · Agua 30 t · Repuestos 2 t · Energía 1 MW |
| Con reciclaje (VID-2.3, fuera del MVP) | Agua −75% |
| Crecimiento | Sólo por **inmigración** (colonizadora COL-1) en el MVP |
| Malestar | Sube si falta alimento/agua/repuestos; sin suministro 60 días → crisis (abandono) |
| Autonomía | Dependiente: el Estado envía suministros; las colonias no generan comercio propio salvo minería |

---

## 9. Programas del MVP (07)

Disponibles con **ADM-2.1**:

| Programa | Parámetros |
|---|---|
| Minería | Cuerpo, recurso, toneladas/mes, modo (propio / contratado / mixto) |
| Relevamiento | Lista de cuerpos o zonas, prioridad, modo |
| Transporte (línea) | Origen, destino, bien, toneladas/mes, modo |

Antes de ADM-2.1 todo es **manual** por nave.

---

## 10. Objetivos sugeridos del MVP

Lista de 02 (opcionales): primer satélite · primera sonda · estación en órbita · relevar la Luna · primera colonia · minería de asteroides · localizar el punto de salto · **cruzarlo (fin del MVP)**.

**Cierre:** al cruzar el punto de salto con EXP-2 se muestra un resumen de la partida (crónica, estadísticas) y la pantalla "Fase 1 completada". El contenido posterior (MVP+: primer contacto) queda para después.

---

## 11. Archivos de datos

```
data/
  bodies.json          # 21 cuerpos + punto de salto + yacimientos
  goods.json           # 16 bienes
  recipes.json         # recetas
  facilities.json      # instalaciones
  techs.json           # 38 + 3 iniciales
  designs.json         # 15 diseños de fábrica
  companies.json       # empresas privadas
  scenario_earth.json  # estado inicial de la Tierra, grupos, presupuesto
  events.json          # eventos sociales simples
  loc/es.json          # textos
  params.json          # parámetros globales (días por semana, mes y año; §0)
```

### 11.1 Cuerpos (`bodies.json`)

| Campo | Contenido |
|---|---|
| `id` | Texto estable (`body.*`) |
| `kind` | `star`, `planet`, `dwarf_planet`, `moon`, `asteroid` o `gas_giant` |
| `parent` | Id del cuerpo padre; **sólo `star` puede no tenerlo** |
| `orbit` | Órbita circular (`type: "circular"`); ausente en la estrella. Radio en `radius_au` (UA) **o** `radius_km` (km), exactamente uno; `period_days` |
| `radius_km` | Radio del cuerpo, en km (> 0) |
| `mass_kg` | Masa, en kg (> 0); obligatorio (1.1) |
| `detect_difficulty` | Entero de 0 a 10 |
| `habitability` | 0–1; ausente si no aplica |
| `deposits` | Yacimientos: `good`, `reserve_mt` (> 0) y `access` (0–1) |

Los valores numéricos deben ser finitos. El cargador rechaza lo que no cumpla, con archivo, id y campo.

Ejemplo (cuerpo):

```json
{
  "id": "body.luna",
  "kind": "moon",
  "parent": "body.tierra",
  "orbit": { "type": "circular", "radius_km": 384400, "period_days": 27.3 },
  "radius_km": 1737,
  "mass_kg": 7.35e22,
  "detect_difficulty": 0,
  "habitability": 0.30,
  "deposits": [
    { "good": "good.silicatos", "reserve_mt": 1000000, "access": 0.9 },
    { "good": "good.hierro", "reserve_mt": 200000, "access": 0.7 },
    { "good": "good.hielo", "reserve_mt": 600, "access": 0.3 }
  ]
}
```

---

## 12. Preguntas abiertas y calibración

- Calibrar el **costo de lanzamiento** frente al presupuesto: con 600 M cr/año se pueden hacer sólo ~20 lanzamientos de LV-1 por año al inicio; ¿es el ritmo deseado?
- Calibrar PI/mes y crecimiento de laboratorios: objetivo de **25–30 años** para la ruta completa.
- ¿Los costos de diseños y de instalaciones sostienen un juego interesante en las eras 1–2?
- ¿Se agrega ADM-0.2 al MVP (más científicos) o se supone un flujo fijo?
- Valores de rendimiento de las minas, energía y consumo de colonias.
- Cantidad de eventos sociales simples y su impacto en el apoyo público.
- Decisión pendiente: ¿se incluyen en el MVP las lunas de Júpiter y Titán como "sólo observables", o se esconden hasta el MVP+?

## Conexiones
02 (Fase 1) · 04 (relevamiento y recursos) · 05 (colonias, regiones) · 07 (diseños de fábrica, programas) · 13 (tecnologías) · 17 (ruta crítica) · 18 (bienes y recetas) · 19 (incentivos) · 21 (MVP y arquitectura).
