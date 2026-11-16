# 11 — Guerra, combate y seguridad

> **Estado:** v0.2 con parámetros numéricos. **Todos los valores son orientativos** y se calibran con el prototipo de combate (Etapa B, ver 17).

## Niveles

| Nivel | Escala | Referente |
|---|---|---|
| **Espacial táctico** | Naves y flotas en un sistema | Aurora 4X, mejorado |
| **Seguridad y patrullaje** | Zonas, sistemas, rutas | Nuevo |
| **Terrestre** | Formaciones por región (abstracto) | Grand strategy |
| **Estratégico** | La guerra como sistema: economía, moral, diplomacia | Victoria, HOI |

**Consistencia:** la resolución automática usa las mismas estadísticas que el combate controlado (01).

---

## Objetivo de diseño del combate

En Aurora una nave pequeña puede aparecer de la nada y destruir tu nave mientras esperás muchos ticks sin poder hacer nada. Acá el jugador debe:

1. **Saber más** de lo que pasa (detección anticipada, estimaciones).
2. Tener **opciones reales** (huir, resistir, pedir ayuda), aunque la ayuda tarde.
3. No **perder tiempo** en combates ya decididos.
4. Poder **prevenir** con infraestructura de seguridad.

---

## Unidades y escala (orientativas)

| Magnitud | Unidad | Referencia |
|---|---|---|
| Distancia | Millones de km (Mkm) | 1 UA ≈ 150 Mkm |
| Velocidad | km/s | Nave del sistema: 2.000–8.000 km/s |
| Tiempo | Incrementos 5 s · 1 h · 1 día · 5 días · 30 días | 01 |

Con 5.000 km/s una nave recorre ~432 Mkm por día: cruzar un sistema de 40 UA lleva ~2 semanas.

---

## Detección (decidido: sensibilidad vs. firma)

Cada sensor tiene una **sensibilidad (S)** y cada nave una **firma (F)** en tres canales: **térmica** (motores y reactor), **electromagnética** (comunicaciones y sensores activos) y **tamaño/geométrica**. Alcance orientativo:

```
alcance_detección (Mkm) ≈ 0,5 × √(S_sensor × F_objetivo)
```

### Tipos de sensor
| Tipo | Detecta | ¿Revela al emisor? |
|---|---|---|
| **Pasivo térmico** | Motores y reactores encendidos | No |
| **Pasivo EM** | Emisiones (comunicaciones, radares) | No |
| **Activo (radar)** | Tamaño y posición, incluso naves frías | **Sí** |

### Ejemplos de sensibilidad (S) y firma (F)
| Elemento | S / F |
|---|---|
| Estación planetaria | S = 200 |
| Puesto de vigilancia | S = 100 |
| Nave de patrulla | S = 30 |
| Nave civil | S = 5 |
| Gran nave, motor a máxima | F ≈ 500 |
| Nave mediana, motor encendido | F ≈ 50 |
| Nave mediana, motor apagado (deriva) | F ≈ 5 |
| Nave pequeña fría | F ≈ 1 |

Ejemplos de alcance: un puesto (S=100) ve a una gran nave a plena potencia a ~112 Mkm y a una nave mediana en deriva a ~11 Mkm; una patrulla (S=30) las ve a ~61 y ~6 Mkm.

### Calidad de contacto
Desconocido → tipo → clase → composición estimada. La calidad depende de la intensidad de la señal y de los **sensores especializados**. Sigilo, ECM y deriva fría reducen la calidad.

---

## Movimiento

Velocidad y combustible (estilo Aurora, sin delta-v). Persecuciones e intercepciones con costo de combustible. Una nave puede **apagar motores** para reducir su firma, perdiendo la capacidad de maniobrar.

## Armas (alcances orientativos)

| Arma | Alcance | Notas |
|---|---|---|
| Cañón de riel / cinético | ~0,15 Mkm | Daño alto contra blindaje |
| Láser / haz | ~0,4 Mkm | Daño cae con la distancia |
| Defensa puntual | ~0,05 Mkm | Contra misiles y drones |
| **Misil** | ~20 Mkm (vel. ~12.000 km/s) | Salvas vs. defensa puntual; consumen mucha logística |
| Caza/dron (ver abajo) | ~5–15 Mkm de acción | Alcance limitado por combustible |

## Cazas y drones portados (decidido)

Hangares en **portaaviones o naves con hangar** (capacidad aproximada: 1 caza por 250 t de hangar). Usan el sistema de diseño (07).

| Unidad | Velocidad | Autonomía | Armamento | Rol |
|---|---|---|---|---|
| **Caza ligero** | ~9.000 km/s | ~8 h | Misiles anti-nave / armas ligeras | Ataque, escolta, defensa puntual |
| **Dron de vigilancia** | ~3.000 km/s | ~30 días | Sensor | Reconocimiento, extender cobertura |
| **Dron de ataque** | ~8.000 km/s | ~2 h | Carga explosiva | Kamikaze contra objetivos grandes |

Los cazas dependen del hangar y de repuestos; los drones de vigilancia pueden desplegarse como **picket** en rutas y puntos de salto.

## Daño

Capas de blindaje → componentes internos (un impacto puede destruir el motor, los sensores o el soporte vital), bajas de tripulación, incendios, despresurización.

## Mantenimiento
Las naves se deterioran lejos de bases (reloj de mantenimiento estilo Aurora, 07).

---

## Mecanismos de combate dinámico

### 1. Alerta de contacto anticipada
Los sensores, patrullas y **puestos de vigilancia** emiten alertas con: origen probable, **hora estimada de llegada (ETA)**, composición estimada y nivel de amenaza.

```
tiempo_de_aviso ≈ alcance_detección ÷ velocidad_enemiga
```

Objetivos de diseño: **colonias ≥ 12 h** de aviso; **capital ≥ 3 días** (se logra con puestos en los puntos de salto y patrullas adelantadas). Sin sensores, el contacto sigue siendo posible, pero es por falta de cobertura (no "aparece de la nada").

### 2. Doctrinas y reglas de enfrentamiento (ROE)
Cada flota/patrulla/puesto tiene una **doctrina** permanente que actúa aunque el tiempo corra rápido:

| Parámetro | Opciones |
|---|---|
| Postura | Agresiva · Defensiva · Evasiva · Neutral |
| Compromiso | Atacar a todo hostil · sólo si está en rango · sólo ante ataque |
| Umbral de retirada | % de daño/fuerza a partir del cual rompe contacto (por defecto 40 % de poder perdido) |
| Retirada | Hacia el punto de salto/base más cercano · hacia la flota amiga más cercana |
| Persecución | No / hasta N saltos / hasta quedarse sin combustible |
| Pedido de ayuda | Automático al detectar amenaza superior |
| Prioridad de objetivos | Armas · motores · sensores · logística |

Las doctrinas se heredan por plantilla y se pueden sobreescribir por flota. El almirante (06) las **interpreta**: un almirante cobarde o temerario puede desviarse de la doctrina.

### 3. Ventanas de decisión (decidido: por umbral configurable)

El juego **baja la velocidad** del avance y abre una ventana de decisión cuando la **amenaza o lo que está en juego supera un umbral que define el jugador**. Umbrales por defecto (cualquiera de ellos dispara la ventana):

- La flota en riesgo vale ≥ **5 %** del valor total de tu flota.
- Una colonia con ≥ **100.000 habitantes** está amenazada.
- Un contacto hostil con poder ≥ **50 %** del poder de la fuerza propia más cercana.
- Un contacto no identificado a < **100 Mkm** de un punto de salto propio o una colonia.

El resto de los contactos lo resuelven las **doctrinas** y se informan en un resumen.

Opciones en la ventana:
- **Huir** (con probabilidad de éxito y combustible estimados).
- **Resistir** (con estimación del resultado en rango).
- **Pedir refuerzos** (con ETA de la ayuda más cercana).
- **Cambiar doctrina** / reposicionar.
- **Dejar que la doctrina actúe.**

Como el tiempo avanza por incrementos, la ventana es una **pausa**: el jugador no está bajo presión de tiempo real.

### 4. Resolución rápida de combates desiguales (decidido: 4:1, configurable)

```
poder_combate = Σ (daño_efectivo × factor_alcance) × integridad_blindaje × factor_sensores
```

- Si `poder_bando_A ≥ 4 × poder_bando_B`, el combate se **resuelve en segundos** con un resumen: bajas, daños, qué pasó y por qué.
- El **umbral es configurable** (3:1 más agresivo, 6:1 más conservador) y el jugador puede **forzar el modo detallado**.
- Resultado típico: el bando superior pierde 0–10 % de su fuerza; el inferior, 90–100 %, con variación aleatoria.
- **No se aplica la resolución rápida** si hay naves con nombre/insignia en juego, una colonia importante bajo ataque, o el jugador lo pide.

### 5. Refuerzos, escape y rescate
- **Escape:** huir hacia un punto de salto (si hay) o fuera del alcance enemigo. Probabilidad orientativa:

```
p_escape ≈ clamp( 0,5 + 0,8 × (v_propia − v_enemiga) / v_enemiga − penalización_por_distancia_a_salida , 0 , 0,95 )
```

- **Refuerzos:** pedir ayuda a flotas, patrullas o bases cercanas. Las distancias son grandes y la ayuda puede llegar tarde, pero **la opción siempre existe** y su resultado se cuenta en el informe.
- **Rescate:** naves dañadas pueden ser remolcadas o evacuadas.

### 6. Ralentización automática
Con un contacto hostil a menos de **3 × el alcance máximo de armas** de una unidad propia, el incremento baja automáticamente a **5 s**; al terminar el contacto, vuelve al incremento anterior.

---

## Seguridad, patrullaje y defensa preventiva

### Componentes
- **Puestos de vigilancia automáticos:** estaciones o plataformas con sensores y/o defensas, de bajo costo y mantenimiento.
- **Patrullas:** flotas con rutas y horarios configurables, reportan contactos y reaccionan según su doctrina.
- **Sensores:** cobertura por radio y tipo (pasivos, activos, térmicos).
- **Minas en puntos de salto:** campos con densidad y tipo; tienen **IFF** (no dañan a los amigos), se pueden **barrer** por el enemigo y requieren reposición.
- **Defensas fijas:** plataformas y fortificaciones en puntos de salto y planetas.
- **Bases de respuesta rápida:** escuadrones con alcance a varias zonas.

### Nivel de seguridad por sistema/zona (0–100)

```
seguridad = 30 × Cobertura + 25 × Patrullas + 20 × Respuesta + 15 × Defensas + 10 × Minas
```

Cada componente se normaliza entre 0 y 1:

| Componente | Cómo se calcula (orientativo) |
|---|---|
| **Cobertura** | Fracción de rutas y puntos clave del sistema dentro del alcance de sensores contra una nave de firma 5 |
| **Patrullas** | `min(1, visitas_por_mes × poder_flota / (30 × poder_de_referencia_zona))` |
| **Respuesta** | `clamp((72 − t_respuesta_h) / 66, 0, 1)`; 0 si la fuerza armada más cercana tarda ≥ 72 h, 1 si ≤ 6 h |
| **Defensas** | `poder_defensas_fijas / poder_de_amenaza_de_referencia`, máx. 1 |
| **Minas** | Densidad de minas × fracción de puntos de salto cubiertos |

### Efectos del nivel de seguridad

| Nivel | Estado | Pérdida esperada por viaje | Prima de seguros | Comercio privado |
|---|---|---|---|---|
| 0–20 | **Peligrosa** (zona pirata) | ~20 % | ×3,0 | Casi nulo |
| 21–40 | Vulnerable | ~8 % | ×1,8 | Muy reducido |
| 41–60 | Moderada | ~3 % | ×1,2 | Normal-reducido |
| 61–80 | **Segura** | ~1 % | ×1,0 | Normal |
| 81–100 | Muy segura | ~0,2 % | ×0,9 | Alto, atrae inversión |

Además:
- Riesgo de **incursiones y piratería**: `p_mensual ≈ base × (1 − seguridad/100)² × atractivo_de_la_zona`.
- Influye en el **malestar** y la **lealtad** de colonias (05) y en la **inversión privada** (03, 19).
- Hay zonas **seguras (no del todo)** y zonas peligrosas; subir la seguridad cuesta recursos y mantenimiento.

### Costos orientativos

| Elemento | Construcción | Mantenimiento anual |
|---|---|---|
| Puesto de vigilancia básico | 1,2 M cr | ~2 % (+ pocos repuestos) |
| Puesto de vigilancia avanzado | 6 M cr | ~3 % |
| Plataforma defensiva | 15 M cr | ~3 % |
| Patrulla (3 naves ligeras) | 30 M cr | ~5 % + combustible |
| Campo de minas (200 minas) | ~0,5 M cr | Reposición ~10 % anual |
| Dron de vigilancia (picket) | 0,4 M cr | ~5 % |

Los costos de mantenimiento se expresan como **repuestos y suministros** (18) y tienen que ser **transportados**, lo que conecta seguridad con logística (04).

---

## Origen de las amenazas

Toda amenaza viene de alguna parte (01):

- **Facciones y flotas simuladas** con bases y logística reales.
- **Piratas y saqueadores con bases reales** en zonas de baja seguridad (típicamente 2–6 naves ligeras por grupo). Atacan lo **rentable y débil**; se pueden eliminar (asaltando su base) o negociar con ellos (10).
- **Amenazas ambientales o precursoras:** guardianes, ruinas activas, fenómenos, restos del colapso.
- **Rebeldías internas:** colonias o flotas propias que se rebelan por malestar (05, 09).
- **Incursiones por puntos de salto desconocidos** (dificultad opcional): civilizaciones (o entidades) abren un punto de salto en un lugar no conocido e inician una incursión desde allí.

### Incursiones por puntos de salto desconocidos (orientativo)

- **No es instantáneo:** el juego avisa de una **anomalía detectada** en una zona, con **posibles interpretaciones** (incursión, fenómeno natural, otra cosa).
- **Tiempo de aviso:** entre **3 y 30 días**, según la calidad de los sensores gravimétricos y de la cobertura (04, 14).
- **Probabilidad de identificación correcta** de la anomalía: ~30 % con sensores básicos, hasta ~90 % con relevamiento avanzado.
- **Frecuencia por nivel de dificultad:** Desactivado · Rara (~1 cada 5 años de juego) · Normal (~1 cada 2 años) · Frecuente (~1 por año).
- **Fuerza de la incursión:** proporcional al poder del jugador (×0,5 a ×1,5 según dificultad), siempre visible en los parámetros (01).

---

## Guerra terrestre (abstracta)

- **Formaciones** (de pelotón a ejército) con composición: infantería, blindados, artillería, apoyo, ingenieros, logística. Diseñadas por el jugador (07).
- Atributos: equipo, entrenamiento, experiencia, moral, organización, suministro.
- **Líderes** con rasgos (06).
- Combate por **región** con modelo de desgaste; fortificaciones, asedios, ocupación; el apoyo orbital pesa mucho.
- Sin mapa táctico ni control de unidades individuales.

## Asaltos planetarios

- Superioridad orbital → desembarco (capacidad de transporte limitada).
- **Bombardeo orbital:** de precisión vs. indiscriminado; el indiscriminado es atrocidad con consecuencias.
- Asedio de cúpulas y hábitats: cortar el soporte vital genera crisis humanitarias.
- **Conquista:** ocupar regiones da recursos, población, tecnología e instalaciones (útil para el estilo imperialista, 03), con rebeliones y malestar (05, 09).

---

## Movilización y economía de guerra

- **Conscripción** reduce mano de obra.
- Conversión industrial, racionamiento, bonos de guerra, impresión de dinero (03).
- **Cansancio de guerra** por pop: bajas, escasez, distancia, justificación percibida.

## Moral
Moral de unidades + **moral del frente interno**. Una guerra puede ganarse en el campo y perderse en casa.

## Guerra irregular
Insurgencia, guerrillas y sabotaje (tratados de forma sobria y abstracta) · piratería y corsarios con patente · **contrainsurgencia:** ganar apoyo de la población vs. represión (trade-off con el malestar, 05).

## Mercenarios
Compañías contratables; lealtad al pago; pueden cambiar de bando si les pagan más o si el cliente se vuelve insolvente.

## Normas y crímenes de guerra
Atrocidades (bombardear civiles, ejecutar prisioneros, armas biológicas) se **registran**, afectan reputación, malestar y agravios, y generan leyendas negras. Las normas varían por cultura.

## Prisioneros
Intercambio, rescate, reclutamiento, trabajo forzado (si el régimen lo permite, 09), juicios.

## Fin de la guerra
Tratados de paz, ocupación, reconstrucción, refugiados, **veteranos**, monumentos, revanchismo.

---

## Conexiones

03 (seguros, economía de guerra) · 04 (suministro, rutas) · 05 (malestar, cansancio) · 06 (almirantes) · 07 (diseño militar, hangares) · 09 (golpes, guerras civiles) · 10 (casus belli, paz) · 13 (sensores, minas, puntos de salto) · 14 (amenazas generadas) · 15 (batallas famosas) · 16 (alertas, ventanas de decisión) · 18 (bienes militares, repuestos) · 19 (aduanas, contrabando).

## Preguntas abiertas

- Calibrar `poder_combate` y el umbral 4:1 con corridas automáticas contra el combate detallado.
- ¿Cómo se explican en el resumen las causas de un resultado de resolución rápida?
- Detalle de armas, blindaje y daño por componente (tablas del diseñador, 07).
- Valores de sensibilidad/firma por tecnología y su evolución (13).
- Fuerza y comportamiento de piratas por zona y su relación con zonas francas (19).
