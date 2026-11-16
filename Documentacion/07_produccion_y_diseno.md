# 07 — Producción, industria y diseño

> v0.3 · Agrega diseños de fábrica, construcción naval privada, **programas y contratos** (órdenes masivas), y hangares/cazas/drones/minas (11). Números iniciales, a calibrar.

## Cadenas productivas

Del crudo al producto final, con pasos intermedios. Ejemplos:

```
Mineral de hierro ─► Acero ─► Placas de blindaje ─► Casco de nave
Arena silícea + Tierras raras ─► Semiconductores ─► Electrónica ─► Sensores
Agua ─► (electrólisis) ─► Hidrógeno + Oxígeno ─► Combustible / Soporte vital
Cultivos ─► Alimento procesado ─► Raciones (civiles / militares / exportación)
Biomasa + Químicos ─► Fármacos ─► Medicina
```

## Recetas

Cada receta define: **insumos**, **productos**, **subproductos** (residuos, contaminación, calor), **instalación requerida**, **energía**, **tiempo** y **mano de obra/habilidad**. La **calidad** depende de la tecnología, la instalación, la calidad de los materiales y el personal.

Las recetas se ejecutan en **instalaciones** (minas, refinerías, fábricas, astilleros), con capacidad, eficiencia y mantenimiento. Pueden ser **estatales o privadas** (03). El detalle de bienes y recetas está en 18.

---

## Materiales con propiedades

Cada material tiene propiedades físicas: densidad · dureza · tenacidad · resistencia a la tracción · resistencia térmica · conductividad · resistencia a la radiación · reactividad.

**Las estadísticas de los componentes se derivan de los materiales:**
- `masa = densidad × volumen`
- `protección del blindaje = f(dureza, tenacidad, espesor)`

Aleaciones y compuestos se investigan (13).

---

## Calidad, durabilidad y mantenimiento

- **Niveles de calidad** de componentes: estándar · bueno · excelente · avanzado · precursor.
- **Mantenimiento** (inspirado en Aurora): naves, instalaciones y maquinaria consumen repuestos y trabajo. Es un drenaje logístico constante y central; sin él, aumentan las fallas.
- **Reparación:** requiere piezas, instalaciones y tiempo (talleres, tenders, astilleros).

---

## Escalas de producción

| Escala | Descripción | Rendimiento | Mano de obra | Capital/energía |
|---|---|---|---|---|
| Taller | Producción pequeña, muy local | Bajo | Alta | Bajo |
| Fábrica | Línea con obreros | Alto | Media | Alto |
| Complejo automatizado | Máquinas con supervisión | Muy alto | Baja | Muy alto |
| Industria orbital / megafábricas | Gran escala en el espacio | Enorme | Muy baja | Enorme |

- La **automatización** desplaza trabajadores → efectos en desempleo y política (03, 05, 09).
- La producción se gestiona como **capacidad** de instalaciones, no pieza por pieza.

---

## Sistema de diseño

Los objetos complejos se **diseñan** con componentes modulares:

- **Naves** (militares, de carga, de relevamiento, de colonización, tenders, de minería, de investigación).
- **Estaciones**, satélites, sondas y plataformas defensivas.
- **Drones** (vigilancia, ataque; 11).
- **Unidades terrestres** (formaciones con componentes: armas, blindaje, movilidad, apoyo).
- **Instalaciones** de colonia y módulos de hábitat.
- **Puestos de vigilancia y minas** (11).

### Diseño de naves (inspirado en Aurora)

Componentes:
- **Casco** (clase/tamaño) y **blindaje** (material, capas).
- **Motores** (tipo, empuje, eficiencia) y **combustible**.
- **Reactor** / fuente de energía y **radiadores** (el calor es una restricción real).
- **Sensores** (activos/pasivos, térmicos, EM) y **comunicaciones**.
- **Armas** y defensa puntual; **ECM/ECCM**; **escudos**.
- **Tripulación**, alojamiento, soporte vital.
- Carga, **hangares**, módulos especiales (minería, colonización, relevamiento, investigación, **construcción de puntos de salto**).
- Almacenamiento de repuestos (autonomía de mantenimiento).

**Estadísticas derivadas:** masa, velocidad, autonomía, firma (detectabilidad), tripulación necesaria, costo, tiempo de construcción, carga de mantenimiento.

El diseñador valida restricciones y muestra **compromisos** (más blindaje = menos velocidad/autonomía).

### Hangares, cazas, drones y minas (decidido, 11)
- **Hangar:** se mide en toneladas; ~**250 t por caza**. Los cazas y drones se **portan** en naves o estaciones.
- **Cazas:** velocidad alta y autonomía corta (horas); deben regresar a reabastecerse.
- **Drones de vigilancia** (autonomía de días a semanas) y **de ataque** (corta autonomía, alta velocidad); se despliegan en puntos de salto, rutas o patrullas.
- **Minas:** módulos de campo (p. ej. 200 minas) que se despliegan en puntos de salto; requieren **reposición** periódica.
- Los números concretos están en 11.

### Astilleros
Capacidad por dársena, refits, producción en serie. Los **astilleros privados** construyen **naves civiles** (cargueros, mineras, colonizadoras, de relevamiento); las **militares y estratégicas** son estatales salvo en régimen corporativo (03).

### Planos (blueprints)
- Son **información**: se pueden vender, licenciar, robar o filtrar.
- **Ingeniería inversa** sobre equipo capturado o restos → conocimiento parcial (13).

---

## Diseños de fábrica

Para ayudar a principiantes el juego incluye **diseños prefabricados** de naves, estaciones, satélites, sondas y drones (02):

- **Genéricos y subóptimos** a propósito: rinden aprox. **70–80%** de un diseño optimizado con la misma tecnología (≈ +20% de masa muerta, −20% de eficiencia de combustible, sin especialización).
- Se **desbloquean con la tecnología** que los habilita (13); no son gratis.
- Se pueden **clonar y editar** para crear diseños propios, o usar tal cual.
- **No se actualizan solos:** con una tecnología nueva aparece una nueva versión de fábrica, también subóptima.
- **Nomenclatura:** familias por rol y era (LV = lanzador, SAT = satélite, SND = sonda, EST = estación, TRN = transporte, MIN = minera, REL = relevamiento, COL = colonizadora, EXP = exploración).

### Catálogo inicial de la Fase 1

| Era | Diseño | Tecnología requerida | Rol / uso | Capacidad / valor aproximado |
|---|---|---|---|---|
| 0 | **LV-1 Lanzador ligero** | PRO-0.1 | Subir carga a órbita baja | 2 t a órbita baja |
| 0 | **SAT-O1 Satélite de observación** | PRO-0.1, SEN-0.1 | Mejora los datos (Sospecha→mejor rango) | Alcance óptico medio |
| 0 | **SAT-C1 Satélite de comunicaciones** | COM-0.1 | Red de comunicaciones | Cobertura global |
| 0 | **CAP-1 Cápsula tripulada** | VID-0.1 | Misiones tripuladas a órbita | 3 tripulantes |
| 1 | **SND-1 Sonda de sobrevuelo** | PRO-1.1, SEN-1.2 | Observar cuerpos (Observado) | Un cuerpo por misión |
| 1 | **SND-2 Sonda orbital** | SEN-1.2, MIN-1.1 | Relevamiento prolongado | Un cuerpo; mejora datos |
| 1 | **EST-1 Estación básica** | VID-1.2 | Base orbital con módulos | 6 tripulantes, 3 módulos |
| 2 | **TRN-1 Transporte orbital** | PRO-1.2 / PRO-2.2 | Carga entre órbitas | ~200 t |
| 2 | **MIN-1 Minera de asteroides** | MIN-2.2 | Extracción en asteroides | ~100 t por viaje |
| 2 | **REL-1 Nave de relevamiento** | SEN-2.1, PRO-2.2 | Relevamiento (Relevado) | Un cuerpo por ciclo |
| 2 | **BAS-1 Base superficial** | VID-2.1 | Hábitat en Luna/planeta | ~200 habitantes |
| 2 | **COL-1 Colonizadora** | VID-2.1, VID-2.3 | Transportar colonos | ~500 colonos |
| 3 | **EXP-1 Exploradora de largo alcance** | PRO-3.1, SEN-2.2 | Exploración profunda y relevamiento gravimétrico | Autonomía larga, sensores |
| 3 | **EXP-2 Nave de salto** | PRO-3.2 | Cruzar puntos de salto | Fin de la Fase 1 |

(Valores iniciales, para guía de balance.)

---

## Construcción naval y naves privadas

A partir de **ADM-1.1** (contratos con privados, 13) aparecen empresas aeroespaciales privadas:

- **Desarrollan sus propios diseños civiles** (cargueros, mineras, colonizadoras, de relevamiento) con su propia tecnología (13).
- El jugador puede:
  - **Comprar naves** a astilleros privados (precio de mercado + margen).
  - **Alquilar servicios** (flete, minería, colonización, relevamiento) por contrato.
  - **Comprar o licenciar la tecnología** (13).
  - **Nacionalizar o encargar** diseños específicos con contratos de obra (03, 19).
- Las naves privadas son **civiles**: no se pueden armar sin un acuerdo especial (03, 09); si hay guerra pueden **requisarse** (con costo político).
- **Militares:** estatales por defecto; en regímenes corporativos las empresas pueden fabricar (03).

---

## Programas y contratos (órdenes masivas)

Para evitar el micromanagement, el jugador gestiona **programas** en lugar de naves una por una (se desbloquea con **ADM-2.1**; COM-3.1 mejora la eficiencia).

### Qué es un programa
Un objetivo con presupuesto y modo de ejecución:

| Programa | Objetivo | Ejemplo |
|---|---|---|
| **Minería** | Extraer X toneladas/mes de un recurso en un cuerpo | "Hierro del asteroide A, 5.000 t/mes" |
| **Colonización** | Establecer y poblar colonias | "Colonia en Luna, 5.000 habitantes en 3 años" |
| **Relevamiento** | Relevar cuerpos o sistemas | "Relevar los 5 cuerpos más cercanos" |
| **Transporte (línea)** | Mover carga entre dos nodos | "Combustible de la Tierra a la Estación 1" |
| **Patrulla / seguridad** | Cobertura de una zona (11, Fase 2) | "Seguridad ≥ 60 en el sistema" |

### Modos de ejecución
- **Propio:** se asignan **grupos de naves** (no una a una) y el sistema reparte las tareas.
- **Contratado:** se paga a **empresas privadas** (por tonelada, por misión, por mes) y ellas aportan naves y personal.
- **Mixto:** parte propia y parte contratada; el sistema reparte según costo y disponibilidad.

### Cómo se resuelven
- El jugador define **objetivo + presupuesto + modo + prioridad**.
- La asignación respeta las restricciones físicas: rutas, combustible, capacidad, riesgo.
- Los programas generan un **informe** (progreso, costo, cuellos de botella) y **alertas** si no se pueden cumplir.
- En la **Fase 1 temprana** hay pocas naves y se controla **manualmente**; los programas aparecen con ADM-2.1 y escalan a la Fase 2 (sistemas enteros).

### Contratos de servicios con privados
Un contrato define: servicio, volumen, precio, duración y penalizaciones. El privado lo acepta si el **retorno esperado** es suficiente (19). Se pueden rescindir con costo; los privados pueden quebrar o incumplir.

---

## Prototipos y curva de aprendizaje

- Las primeras unidades de un diseño nuevo son **menos confiables** y más caras (madurez, 13).
- La experiencia de producción reduce costos y fallas con el tiempo.

## Objetos con historia

Naves insignia, armas legendarias y estaciones **acumulan historia** (quién las construyó, qué batallas vieron). Se vuelven artefactos culturales, botín y motivo de conflicto (15).

---

## Energía

- Toda producción consume energía. Redes con capacidad y pérdidas.
- Fuentes por era: biomasa → combustibles fósiles → solar/eólica/geotérmica → fisión → fusión → exóticas.
- En el espacio: la energía solar depende de la distancia a la estrella; el **calor** debe disiparse con radiadores.

## Residuos y contaminación

Subproductos que hay que almacenar, procesar o liberar al entorno (con costos ambientales y de salud, ver 12).

## Conexiones

02 (diseños de fábrica, Fase 1) · 03 (bienes, empresas, regímenes) · 04 (insumos y transporte) · 06 (directores, científicos) · 11 (diseños militares, hangares, minas) · 12 (contaminación) · 13 (tecnologías desbloquean componentes y diseños) · 15 (objetos con historia) · 16 (UI de programas) · 18 (recetas) · 19 (contratos e incentivos).

## Preguntas abiertas

- Profundidad del diseño de unidades terrestres (¿plantillas o componentes?).
- Cantidad de materiales base (propuesta: ~30–50).
- Reparto de tareas entre propio y contratado en programas mixtos (algoritmo y transparencia).
- Cuánto del programa se muestra al jugador (¿por nave o sólo agregado?).
- Valores concretos de masa, capacidad y costo de los diseños de fábrica.
