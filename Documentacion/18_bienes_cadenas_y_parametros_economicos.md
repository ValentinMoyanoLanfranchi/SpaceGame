# 18 — Bienes, cadenas productivas y parámetros económicos

> **Estado:** borrador numérico v0.2. **Todos los valores son orientativos** y están pensados para calibrarse con el prototipo de economía (Etapa B, ver 17). Complementa a `03_economia.md` (reglas), `04_recursos_y_logistica.md` (transporte) y `19_impuestos_incentivos_y_mercados_grises.md` (impuestos por categoría, incentivos, sustancias y leyes de trabajo).

## Convenciones

- **Masa:** toneladas (t). **Dinero:** créditos (cr), una unidad de referencia común para balancear; cada facción tiene su moneda con tipo de cambio (03).
- **Precio base:** costo/valor de referencia por tonelada en una colonia con acceso normal al recurso y seguridad media. El precio real varía por nodo (stock, transporte, riesgo) y, en los bienes de cambio, por el **índice de su sector**.
- **Perecibilidad:** días antes de que el stock se degrade apreciablemente (— = no perece).
- **Legalidad:** L = legal por defecto, R = regulado (licencia/permiso), P = prohibido en muchas facciones. Cada facción define su propia matriz de legalidad (19).
- **Minerales:** **reales con unos pocos ficticios** (los ficticios son los exóticos y precursores).
- **Sustancias:** **ficticias**. No se modelan sustancias reales ni su producción real.

---

## Lista de bienes del núcleo (~73)

### 1. Crudos metálicos y minerales

| # | Bien | Precio base (cr/t) | Perece | Legal | Notas |
|---|---|---|---|---|---|
| 1 | Mineral de hierro | 20 | — | L | Base de la industria pesada |
| 2 | Bauxita | 30 | — | L | → aluminio |
| 3 | Mineral de cobre | 60 | — | L | Electrónica y redes |
| 4 | Mineral de níquel | 70 | — | L | Aleaciones |
| 5 | Mineral de titanio | 150 | — | L | Casco y blindaje avanzados |
| 6 | Tierras raras | 500 | — | R | Electrónica avanzada, sensores |
| 7 | Platinoides | 2.000 | — | R | Catalizadores, electrónica, joyería |
| 8 | Mineral fisible (U/Th) | 800 | — | R | Reactores; peligroso |
| 9 | Silicatos | 10 | — | L | Construcción, silicio |

### 2. Crudos volátiles y orgánicos

| # | Bien | Precio base | Perece | Legal | Notas |
|---|---|---|---|---|---|
| 10 | Hidrocarburos | 50 | — | L | Si el planeta los tiene; plásticos, químicos |
| 11 | Hielo de agua | 5 | — | L | Agua, hidrógeno, oxígeno |
| 12 | Hidrógeno | 40 | — | L | Combustible, química |
| 13 | Helio-3 | 1.500 | — | R | Fusión; gigantes gaseosos |
| 14 | Volátiles pesados (NH₃, CH₄, N₂) | 25 | — | L | Fertilizantes, combustible, química |
| 15 | Biomasa (cultivos crudos) | 15 | 20 | L | Alimentos, bebidas, papel, sustancias |

### 3. Exóticos (precursores / ficticios)

| # | Bien | Precio base | Perece | Legal | Notas |
|---|---|---|---|---|---|
| 16 | Cristales de presión *(ficticio)* | 3.000 | — | R | Sensores, sistemas especiales, joyería de alta gama |
| 17 | Isótopos raros | 5.000 | — | R | Investigación, armas |
| 18 | Materia precursora *(ficticio)* | 20.000 | — | P | Tecnología precursora; peligrosa e inestable |

### 4. Refinados

| # | Bien | Precio base | Perece | Legal | Notas |
|---|---|---|---|---|---|
| 19 | Acero | 80 | — | L | Estructura, naves, maquinaria |
| 20 | Aluminio | 100 | — | L | Cascos ligeros |
| 21 | Cobre refinado | 150 | — | L | Cables, electrónica |
| 22 | Aleaciones avanzadas | 400 | — | L | Titanio-níquel; blindaje y cascos |
| 23 | Silicio de alta pureza | 200 | — | L | Semiconductores |
| 24 | Polímeros | 90 | — | L | Aislantes, bienes de consumo |
| 25 | Químicos industriales | 120 | — | R | Procesos, fármacos, sustancias |
| 26 | Agua tratada | 15 | — | L | Consumo y soporte vital |
| 27 | Combustible químico | 100 | — | L | Naves del sistema y cohetes |
| 28 | Combustible de fusión | 3.000 | — | R | Naves avanzadas y reactores |
| 29 | Combustible fisible | 1.500 | — | R | Reactores de fisión |
| 30 | Gases de soporte vital | 50 | — | L | O₂, N₂ presurizados |

### 5. Componentes

| # | Bien | Precio base | Perece | Legal | Notas |
|---|---|---|---|---|---|
| 31 | Piezas mecánicas | 250 | — | L | Maquinaria y reparaciones |
| 32 | Electrónica | 600 | — | L | Control, sensores básicos |
| 33 | Semiconductores avanzados | 2.500 | — | R | Sensores y computación avanzados |
| 34 | Baterías y almacenamiento de energía | 700 | — | L | Redes, hábitats |
| 35 | Módulos de soporte vital | 1.200 | — | L | Hábitats, naves |
| 36 | Materiales de construcción | 40 | — | L | Edificios de colonia |
| 37 | **Repuestos y suministros de mantenimiento** | 300 | — | L | **Drenaje logístico central** (07) |
| 38 | Sistemas de propulsión (componentes) | 2.000 | — | L | Motores y toberas |

### 6. Consumo esencial (siempre por nodo)

| # | Bien | Precio base | Perece | Legal | Notas |
|---|---|---|---|---|---|
| 39 | Alimentos básicos | 40 | 30 | L | Necesidad de supervivencia (05) |
| 40 | Alimentos procesados | 90 | 120 | L | Raciones, exportación |
| 41 | Bienes de consumo básicos | 120 | — | L | Enseres y ropa de uso diario |
| 42 | Medicinas | 800 | 365 | L | Salud |

### 7. Bienes de cambio (por sector)

Son los bienes que circulan principalmente **entre privados** y responden a los ingresos y la confianza de los pops. Se agrupan en **sectores**, cada uno con su **índice** (ver "Mercado de los bienes de cambio").

| # | Sector | Bien | Precio base | Perece | Legal | Notas |
|---|---|---|---|---|---|---|
| 43 | Bebidas y gastronomía | Vino | 300 | — | L | Valor cultural |
| 44 | | Cerveza y fermentados | 120 | 90 | L | Consumo masivo |
| 45 | | Licores | 600 | — | L/R | Impuesto selectivo típico |
| 46 | | Gastronomía fina | 1.200 | 60 | L | Élites |
| 47 | Joyería y arte | Joyas | 6.000 | — | L | Valor refugio; alta sensibilidad al ciclo |
| 48 | | Arte y artesanías | 3.000 | — | L | Trabajo calificado; valor cultural |
| 49 | Papelería y cultura | Papel | 100 | — | L | Administración, educación |
| 50 | | Libros e impresos | 400 | — | L | Educación y cultura |
| 51 | | Medios de entretenimiento | 600 | — | L | Ocio |
| 52 | Tecnología de consumo | Dispositivos personales | 6.000 | — | L | Celulares, tabletas; obsolescencia rápida |
| 53 | | Pantallas | 3.000 | — | L | Televisores y monitores |
| 54 | | Computadoras y consolas | 5.000 | — | L | Hogar y ocio |
| 55 | | Electrodomésticos | 1.200 | — | L | Comodidad del hogar |
| 56 | Hogar y moda | Moda | 900 | — | L | Ropa de moda |
| 57 | | Muebles y decoración | 500 | — | L | Vivienda |
| 58 | Sustancias | Tabaco | 700 | — | L/R | Impuesto selectivo típico |
| 59 | | **Vigorán** *(estimulante, ficticio)* | 4.000 | — | R | Productividad ↑, adicción media (19) |
| 60 | | **Sosiego** *(sedante, ficticio)* | 3.500 | — | R | Malestar ↓, productividad ↓ (19) |
| 61 | | **Bruma** *(recreativa, ficticia)* | 5.000 | — | P | Ocio; riesgo social (19) |
| 62 | | **Lumbre** *(narcótico, ficticio)* | 12.000 | — | P | Alta adicción, alta prima (19) |

### 8. Bienes de capital

| # | Bien | Precio base | Perece | Legal | Notas |
|---|---|---|---|---|---|
| 63 | Maquinaria industrial | 900 | — | L | Fábricas y refinerías |
| 64 | Equipos de minería | 800 | — | L | Minas y plataformas de extracción |
| 65 | Equipos de laboratorio | 1.800 | — | L | Investigación |
| 66 | Módulos prefabricados de hábitat | 2.200 | — | L | Colonias en cuerpos hostiles |

### 9. Militares

| # | Bien | Precio base | Perece | Legal | Notas |
|---|---|---|---|---|---|
| 67 | Munición | 400 | — | R | Armas cinéticas |
| 68 | Misiles | 3.500 | — | R | Naves y defensas |
| 69 | Placas de blindaje | 600 | — | R | Naves y fortificaciones |
| 70 | Equipo terrestre | 1.500 | — | R | Unidades terrestres (07) |
| 71 | Minas espaciales | 2.500 | — | R | Puntos de salto (11) |

### 10. Información (no ocupa masa relevante)

| # | Bien | Valor | Legal | Notas |
|---|---|---|---|---|
| 72 | Datos de relevamiento | Variable | L/R | Yacimientos, puntos de salto, sistemas (04) |
| 73 | Planos y licencias | Variable | R | Diseños y tecnologías (07, 13) |

### Servicios y energía (locales, no transportables)

| Servicio | Notas |
|---|---|
| Energía eléctrica (red) | Por red; capacidad y pérdidas (07) |
| Salud, educación, seguridad, ocio, culto, administración | Cobertura y calidad (05) |

---

## Cadenas productivas del núcleo (ratios orientativos)

Cada receta: insumos → productos, más **energía** y **instalación**.

### Industria base

| Receta | Insumos | Producto | Instalación |
|---|---|---|---|
| Acero | 2,0 t mineral de hierro + 0,2 t hidrocarburos | 1 t acero | Acería |
| Aluminio | 4,0 t bauxita | 1 t aluminio | Refinería de aluminio |
| Cobre refinado | 3,0 t mineral de cobre | 1 t cobre refinado | Refinería |
| Aleaciones avanzadas | 0,5 t titanio + 0,3 t níquel + 0,3 t acero | 1 t aleaciones | Planta de aleaciones |
| Silicio | 3,0 t silicatos | 1 t silicio | Planta de silicio |
| Polímeros | 1,5 t hidrocarburos | 1 t polímeros | Planta química |
| Agua tratada | 1,2 t hielo de agua | 1 t agua | Planta de tratamiento |
| Combustible químico | 1,0 t hidrógeno + 0,5 t gases de soporte | 1 t combustible | Planta de combustible |
| Combustible de fusión | 0,2 t helio-3 + 0,8 t hidrógeno | 1 t combustible | Planta de fusión |
| Gases de soporte vital | 1,0 t hielo de agua (electrólisis) | 0,9 t gases | Planta de electrólisis |
| Piezas mecánicas | 0,6 t acero + 0,2 t aluminio | 1 t piezas | Taller de precisión |
| Electrónica | 0,3 t cobre refinado + 0,2 t silicio + 0,3 t polímeros | 1 t electrónica | Fábrica de electrónica |
| Semiconductores avanzados | 0,3 t silicio + 0,05 t tierras raras + 0,1 t platinoides | 1 t semiconductores | Fábrica de semiconductores |
| Repuestos | 0,4 t piezas + 0,3 t electrónica + 0,3 t aleaciones | 1 t repuestos | Fábrica de repuestos |
| Maquinaria industrial | 0,5 t acero + 0,3 t piezas + 0,2 t electrónica | 1 t maquinaria | Fábrica de maquinaria |
| Módulos de soporte vital | 0,3 t piezas + 0,3 t electrónica + 0,4 t gases | 1 t módulos | Fábrica de módulos |
| Munición | 0,6 t acero + 0,2 t químicos | 1 t munición | Planta de munición |
| Placas de blindaje | 0,7 t aleaciones + 0,3 t acero | 1 t placas | Planta de blindaje |

### Consumo esencial

| Receta | Insumos | Producto | Instalación |
|---|---|---|---|
| Alimentos básicos | 1,5 t biomasa | 1 t alimentos | Procesadora |
| Alimentos procesados | 1,2 t alimentos + 0,2 t polímeros | 1 t procesados | Procesadora |

### Bienes de cambio (nuevas)

| Receta | Insumos | Producto | Instalación |
|---|---|---|---|
| Vino | 1,5 t biomasa | 1 t vino | Bodega |
| Cerveza | 1,2 t biomasa + 0,5 t agua tratada | 1 t cerveza | Cervecería |
| Licores | 2,0 t biomasa | 1 t licores | Destilería |
| Gastronomía fina | 1,2 t alimentos procesados + 0,1 t polímeros | 1 t gastronomía fina | Taller gourmet |
| Joyas | 0,5 t platinoides + 0,1 t cristales de presión | 1 t joyas | Taller de joyería |
| Arte y artesanías | 0,3 t aleaciones/silicatos + trabajo calificado | 1 t arte | Taller de artesanos |
| Papel | 2,0 t biomasa + 0,3 t químicos | 1 t papel | Papelera |
| Libros e impresos | 0,9 t papel + 0,1 t químicos | 1 t libros | Imprenta |
| Medios de entretenimiento | 0,2 t electrónica + 0,3 t polímeros | 1 t medios | Estudio/fábrica de medios |
| Dispositivos personales | 0,4 t electrónica + 0,1 t semiconductores + 0,2 t baterías + 0,3 t polímeros | 1 t dispositivos | Fábrica de electrónica de consumo |
| Pantallas | 0,4 t electrónica + 0,3 t silicio + 0,3 t polímeros | 1 t pantallas | Fábrica de pantallas |
| Computadoras y consolas | 0,5 t electrónica + 0,05 t semiconductores + 0,3 t aluminio + 0,15 t polímeros | 1 t computadoras | Fábrica de electrónica de consumo |
| Electrodomésticos | 0,4 t acero + 0,3 t electrónica + 0,3 t polímeros | 1 t electrodomésticos | Fábrica de línea blanca |
| Moda | 0,7 t biomasa + 0,3 t polímeros | 1 t moda | Taller textil |
| Muebles y decoración | 0,5 t biomasa + 0,3 t acero + 0,2 t polímeros | 1 t muebles | Carpintería/fábrica |
| Tabaco | 1,5 t biomasa | 1 t tabaco | Procesadora de cultivos |
| Vigorán | 3,0 t biomasa + 0,2 t químicos | 1 t Vigorán | Laboratorio químico |
| Sosiego | 2,5 t biomasa + 0,3 t químicos | 1 t Sosiego | Laboratorio químico |
| Bruma | 2,0 t biomasa + 0,1 t químicos | 1 t Bruma | Laboratorio químico |
| Lumbre | 4,0 t biomasa + 0,8 t químicos + 0,1 t hidrocarburos | 1 t Lumbre | Laboratorio químico |

**Mantenimiento:** toda instalación y toda nave consumen **repuestos** por tiempo de uso (07). Tasa orientativa anual: ~1–3 % del valor de la instalación en repuestos.

---

## Mercado de los bienes de cambio (decidido: índice sectorial + ajuste por nodo)

Los bienes esenciales, crudos, refinados, componentes, capital y militares se mueven **por nodo** (stock vs. objetivo). Los **bienes de cambio** se mueven con un **índice por sector** y un ajuste local, para que las crisis y los booms golpeen distinto a cada sector sin simular cada bien por separado:

```
precio_bien(nodo) = precio_base × índice_sector(facción, t) × ajuste_nodo(stock_local, transporte, seguridad)
```

- `índice_sector` (≈ 1,0 en situación normal) se mueve con la **actividad económica agregada** de la facción, ponderada por la **sensibilidad (β)** del sector, más shocks propios (modas, prohibiciones, escasez de insumos), con reversión lenta a 1,0:

```
Δíndice_sector = β_sector × Δactividad_económica + shock_sector − reversión
```

- `ajuste_nodo` es acotado (orientativo ±30 %) y refleja stock local, costo de transporte y seguridad de la ruta (11); permite arbitraje sin simular precios independientes por bien.
- Se simula **un índice por sector por facción** (≈ 6 sectores): costo bajo, efectos visibles.

| Sector | β (sensibilidad al ciclo) | Quién consume más | Notas |
|---|---|---|---|
| Bebidas y gastronomía | 0,8 | Todos los estratos | Estable salvo crisis fuerte; las bebidas cuentan con impuestos selectivos |
| Joyería y arte | 1,6 | Élites | Muy cíclico; también valor refugio en crisis monetarias |
| Papelería y cultura | 0,5 | Clase media, educación | Defensivo; sube con educación estatal |
| Tecnología de consumo | 1,2 | Clase media/élites | Cíclico; ciclos de obsolescencia (por tecnología civil, 13) |
| Hogar y moda | 1,0 | Todos | Sigue el ciclo medio |
| Sustancias | 0,6 | Variable | Demanda inelástica; muy sensible a la **legalidad y la represión** (19) |

**Demanda:** cada pop gasta en cada sector según su estrato e ingreso (propensión por estrato). Los trabajadores gastan más en cerveza y electrodomésticos; las élites, en joyería, arte y gastronomía fina.

---

## Parámetros de mercado (orientativos)

### Precio por nodo (bienes no cíclicos)

```
precio = precio_base × clamp( (stock_objetivo / max(stock_actual, ε)) ^ elasticidad, 0.25, 6.0 )
```

- `stock_objetivo` = consumo esperado × **días de cobertura** (ver tabla).
- El `clamp` evita precios absurdos; los extremos disparan **alertas** y oportunidades de arbitraje.
- Suavizado temporal: el precio se mueve como máximo ~10 % por día.

| Grupo de bienes | Días de cobertura objetivo | Elasticidad |
|---|---|---|
| Alimentos y agua | 30 | 1,2 (urgente) |
| Gases de soporte vital | 30 | 1,5 (crítico) |
| Combustibles | 45 | 1,0 |
| Repuestos y suministros | 60 | 0,8 |
| Minerales y refinados | 60 | 0,7 |
| Componentes | 60 | 0,6 |
| Militares | 90 | 0,3 |

### Transporte y arbitraje
- Un transportista privado sólo hace un viaje si `margen_esperado ≥ 5–10 %` del valor de la carga después de costos y riesgo.
- El **riesgo** se vincula con el nivel de seguridad de la ruta (11) y se refleja en la **prima de seguros** (03).

---

## Empresas: estatales y privadas (decidido)

### Estatales: modelo híbrido
- Por defecto, el jugador **construye y asigna instalaciones** directamente (control fino estilo Aurora).
- Puede **agruparlas en una empresa estatal** con un **director** (notable, 06), **presupuesto** y **directivas** (prioridad de producción, margen objetivo, expansión, reservas estratégicas), delegando la gestión.
- Una empresa con buen director reduce desperdicio y costo administrativo; con uno corrupto o incompetente, lo aumenta (02, 06).

### Privadas: entrada por rentabilidad, con rubros reservados y licencias
- Las empresas privadas **entran donde hay margen**: invierten cuando el **retorno esperado anual** supera un umbral (orientativo: ≥ 8 %) y tienen capital acumulado. Los **impuestos y regalías por categoría** modifican ese retorno (19).
- El jugador define:
  - **Rubros reservados al Estado** (p. ej. combustible de fusión, munición, minas).
  - **Licencias** obligatorias para ciertos cuerpos, recursos o sistemas (tarifa o **regalías**).
  - **Aranceles** a la importación y exportación.
- Las privadas crecen con las ganancias; pueden **quebrar**, fusionarse o volverse **megacorporaciones** con poder político (09).
- Las facciones extranjeras con empresas privadas pueden invertir en tu territorio bajo pactos (10).

---

## Parámetros fiscales por defecto (orientativos)

| Impuesto / instrumento | Rango jugable | Por defecto |
|---|---|---|
| Renta (pops) | 0–40 % | 15 % |
| Consumo (general) | 0–25 % | 8 % |
| **Selectivos** (licores, tabaco, sustancias reguladas) | 0–80 % | 25 % |
| Empresas | 0–40 % | 20 % |
| Regalías de extracción | 0–30 % | 10 % |
| Aranceles | 0–50 % | 5 % |
| Licencias (tarifa anual) | Variable | Según recurso |

Las tasas se pueden **sobrescribir por grupo, sector o bien** con herencia (19). Efecto indicativo: **impuestos altos** reducen inversión privada y comercio; **bajos** reducen recaudación pero aumentan el crecimiento. Evasión y corrupción crecen con la presión fiscal y con la sobreextensión (02, 09).

---

## Ciclo de crecimiento comercial (resumen)

```
más empresas privadas → más comercio → más actividad económica
        → más recaudación → más infraestructura y seguridad
        → mayor atractivo para inversión → más empresas
```

Límites del ciclo: congestión de rutas, desigualdad y malestar (05), dependencia de importaciones, bloqueos de puntos de salto (04), poder político de las corporaciones (09).

---

## Preguntas abiertas

- Calibrar precios base, ratios de recetas, β sectoriales y márgenes con el prototipo de economía.
- ¿Qué bienes forman la **canasta básica** por especie? (05, 08)
- ¿Los exóticos tienen precio de mercado o sólo se intercambian por contratos?
- Tasa de crecimiento máximo de una empresa privada por año y capital mínimo para entrar en un rubro.
- ¿Cuántos sectores de bienes de cambio se agregan por mods o por especie (p. ej. bienes culturales específicos)?

## Conexiones

03 (reglas económicas) · 04 (transporte y logística) · 05 (necesidades de pops) · 07 (recetas, mantenimiento) · 09 (rubros reservados, poder corporativo) · 10 (pactos, aranceles) · 11 (seguridad y primas) · 19 (impuestos, incentivos, sustancias, trabajo).
