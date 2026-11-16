# 04 — Recursos y logística

> v0.3 · Alineado con 02 (estados de conocimiento), 13 (tecnología), 18 (bienes), 22 (datos del MVP).

## Principio

**Todo bien tiene ubicación.** Moverlo cuesta tiempo, capacidad, combustible y riesgo. La logística conecta la economía (03), la producción (07), la población (05) y la guerra (11).

---

## Recursos

### Clasificación

| Grupo | Ejemplos | Notas |
|---|---|---|
| Metálicos comunes | Hierro, aluminio, cobre, níquel | Abundantes pero de calidad variable |
| Estratégicos / raros | Titanio, tierras raras, platinoides, uranio, torio | Concentrados; motivo de conflictos |
| Volátiles | Agua, hidrógeno, nitrógeno, metano, amoníaco, CO₂ | Críticos en el espacio (soporte vital, combustible) |
| Biomasa | Cultivos, ganado, madera, cultivos microbianos | Perecibles; dependen de clima y suelo |
| Energéticos | Combustibles fósiles (si el planeta los tiene), fisibles, deuterio, helio-3 | El helio-3 empuja hacia gigantes gaseosos/lunas |
| Exóticos | Cristales de alta presión ("cristales de presión" en los asteroides ficticios del MVP), materia precursora, isótopos raros | Motores del late-game |
| Ambientales (no extraíbles) | Insolación, viento, geotermia, suelo fértil | Determinan qué fuentes de energía convienen |

### Yacimientos
Atributos:
- **Cantidad** (toneladas).
- **Concentración / ley**.
- **Accesibilidad** (0–1; determina el costo por tonelada, estilo Aurora).
- **Entorno** (superficie, subsuelo, atmósfera, vacío).
- **Conocimiento** (estimación ± error; mejora con prospección).
- **Reclamo** (quién lo reclama y quién lo controla de hecho).
- **Ubicación:** región de un planeta/luna, o cuerpo menor (asteroide, cometa).

A medida que se agota, la accesibilidad baja → costos crecientes. La sobreexplotación degrada el entorno (12). Los yacimientos pueden explotarlos **empresas estatales o privadas** según licencias (03).

### Conocimiento y relevamiento (exploración)

Cada cuerpo, yacimiento y punto de salto tiene un **estado de conocimiento** (02): **Desconocido → Sospecha → Observado → Relevado → Explorado**. Sólo mejora por **acciones** del jugador.

1. **Estimación remota** (telescopios terrestres y orbitales): barata, imprecisa; lleva de Desconocido a Sospecha/Observado. El poder del telescopio frente a la dificultad de detección del cuerpo decide el tiempo (22).
2. **Sondas y sobrevuelo:** Observado → datos más firmes.
3. **Relevamiento geológico** con naves/equipos: precisión media; revela yacimientos (Relevado).
4. **Relevamiento gravimétrico:** revela **puntos de salto** y anomalías. Se hace **por zonas** del sistema (6 zonas por distancia al Sol, ~6 meses por zona en el MVP, 22). Algunos puntos de salto permanecen ocultos hasta que se releva su zona.
5. **Perforación exploratoria / aterrizaje:** precisa y cara (Explorado).

La información de relevamiento es un **bien**: se vende, se roba, se oculta. Hay falsos positivos y yacimientos ocultos.

---

## Modos de transporte por escala

| Modo | Escala | Capacidad | Velocidad | Costos / riesgos |
|---|---|---|---|---|
| Infraestructura regional (abstracta) | Dentro de un planeta | Según capacidad de la región | — | Congestión, sabotaje, catástrofes (12) |
| Lanzadores (cohetes), cañones de masa, ascensor espacial | Superficie ↔ órbita | Variable (MVP: LV-1 2 t, LV-2 15 t) | — | Costo según pozo gravitatorio y lanzador (MVP: 30 M cr y 70 M cr); el cañón de masa sólo sirve para carga resistente |
| Naves del sistema | Entre cuerpos de un sistema | Alta | Días–semanas | Combustible, mantenimiento, piratería (Fase 2) |
| Naves interestelares (tránsito por puntos de salto) | Entre sistemas | Variable | Días–semanas | Cuellos de botella, minas, bloqueos, piratería |
| Ductos / cintas orbitales | Punto a punto | Flujo continuo | — | Fijos; vulnerables a sabotaje |

> El transporte **dentro de un planeta** se abstrae: no hay mapa de superficie. **En el MVP la Tierra es un único nodo** sin regiones internas (05, 22): todo lo que está en ella se considera conectado y la logística terrestre no se simula. Las regiones con capacidad logística propia se reservan para colonias grandes y para después del MVP.

---

## Infraestructura

- Puertos espaciales, depósitos, estaciones de reabastecimiento, astilleros, relés de comunicaciones, sensores y puestos de vigilancia, **puntos de salto propios** (13).
- **Capacidad limitada** (toneladas/día) → congestión.
- **Mantenimiento:** consume trabajo, repuestos y recursos; sin mantenimiento se degrada.
- **Puntos de salto construidos:** exigen mantenimiento logístico constante (recursos, energía, personal). **Decidido:** son una **inversión media y escalable** (se amplían con tecnología y con más gasto), no un megaproyecto prohibitivo; elegir dónde ponerlos sigue siendo una decisión estratégica.
- Puede ser destruida (guerra, catástrofe, sabotaje).

---

## Red logística (modelo)

- **Grafo:** nodos (colonias, depósitos, estaciones) y aristas (rutas con tiempo, capacidad, costo, riesgo). Los puntos de salto son aristas entre sistemas.
- **Despacho heurístico** (no optimización global): cada nodo publica demandas con prioridad; despachadores asignan transportes disponibles.
- **Prioridades:** Crítica (soporte vital, combate) · Alta · Normal · Baja.

### Quién transporta
- **Flota estatal** (logística y suministro militar, importaciones estatales).
- **Empresas privadas de transporte** que responden a las órdenes de compra/venta de los mercados (03), incluidas las **aeroespaciales** que ofrecen lanzamiento y flete como servicio.
- **Contratos de flete** del Estado a privados.

### Órdenes logísticas del jugador

| Tipo | Ejemplo |
|---|---|
| Manual | "Llevar 50 t de acero de A a B." |
| Ruta permanente | "Cada 10 días, hasta 500 t de agua de Luna-2 a Estación Alfa." |
| Umbral (pull) | "Mantener entre 200 y 800 de alimento en Colonia X." |
| Importación estatal | "Comprar en el sistema Y hasta N unidades y entregarlas en Z." |
| Contrato | Publicar flete con pago; transportistas privados ofertan. Más barato, menos confiable. |
| **Programa / orden masiva** | "Mantener N cargueros en la ruta Tierra–Luna" o "pagar a civiles por X toneladas al mes": una orden única que se reparte sin microgestión (07) |
| Delegada | La oficina logística del gobernador gestiona según políticas. |

### Almacenamiento
Tipos: depósito, refrigerado, tanques presurizados, materiales peligrosos, bóveda segura. Riesgos: deterioro, robo, incendio, explosión.

---

## Movimiento y combustible (estilo Aurora)

- **Órbitas en rieles:** los cuerpos siguen órbitas predecibles (**circulares en el MVP**, 21); no hay delta-v por maniobra.
- Las naves tienen **velocidad y autonomía de combustible** (consumo por distancia).
- Tiempo de viaje = distancia / velocidad (considerando la posición orbital actual).
- Depósitos de combustible y **estaciones de reabastecimiento** son nodos estratégicos.
- **Pozo gravitatorio:** lo que cuesta es **lanzar desde superficie**, no maniobrar en órbita; incentiva industria orbital y minería en cuerpos de baja gravedad. En la Fase 1 el costo de lanzamiento es el cuello de botella económico principal (03).

## Logística interestelar

- Viajes de días a semanas; líneas de suministro largas y vulnerables.
- Las colonias necesitan **reservas de contingencia** y semi-autosuficiencia.
- Los **puntos de salto** son cuellos de botella: se fortifican, patrullan, minan o bloquean (11).
- El **tránsito** por un punto de salto tiene costo/tiempo; los puntos propios pueden ampliarse con tecnología.

---

## Logística militar

- Consumo: combustible, munición, repuestos, alimento, personal.
- **Radio de suministro** desde depósitos y tenders; fuera de suministro → desgaste, baja moral, averías.
- Naves logísticas (tanqueros, cargueros de munición, talleres móviles, naves de reparación).
- Atacar las líneas de suministro enemigas es una estrategia central.
- **Patrullas y puestos de vigilancia** tienen un costo de mantenimiento (11).

## Riesgos

Piratería (desde la Fase 2; pérdida esperada por ruta, relacionada con el nivel de seguridad), accidentes (en la Fase 1: fallos de lanzamiento y de misión), catástrofes, sabotaje, minas, **merma** por corrupción. Mitigación: escoltas, rutas alternativas, seguros, patrullas, mejores leyes.

---

## Visualización

- Mapa de flujos por sistema y entre sistemas (grosor = volumen; color = tipo de bien).
- Cuellos de botella resaltados.
- Alertas de abastecimiento: *"Colonia X sin agua en 6 días"*.
- Rutas con **nivel de seguridad** y antigüedad de la información.
- Estado de conocimiento de cada cuerpo visible en el mapa (16).

## Conexiones

02 (estados de conocimiento) · 03 (precios dependen del transporte, costo de lanzamiento) · 05 (necesidades) · 07 (insumos, naves, programas) · 11 (suministro y seguridad) · 12 (catástrofes cortan rutas) · 13 (tecnologías de transporte, puntos de salto propios) · 14 (geografía) · 18 (bienes) · 22 (datos del MVP: cuerpos, zonas gravimétricas, lanzadores).

## Preguntas abiertas

- Restricciones de tránsito por puntos de salto (¿tamaño máximo de nave, capacidad por día?).
- ¿Cuánto automatizar por defecto sin quitarle juego al jugador?
- Cuántos puntos de salto permanecen ocultos en universos generados (el MVP tiene uno oculto a 14 UA, 22).
- Alcance, costo y mantenimiento exactos de los puntos de salto propios (ver 13).
