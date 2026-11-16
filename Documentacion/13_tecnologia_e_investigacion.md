# 13 — Tecnología e investigación

> v0.3 · Sistema de investigación por proyectos, estructura por campos y niveles, y **lista inicial de 60 tecnologías para la Fase 1** (eras 0–3). Todos los números (costos, ritmos) son valores iniciales a calibrar con prototipos.

## Principios

1. **El conocimiento es físico.** Vive en personas (científicos, ingenieros) y en archivos. Puede perderse, robarse o difundirse.
2. **Árbol visible y determinista.** Campos con niveles y tecnologías con prerrequisitos explícitos; sin opciones al azar. La habilidad del científico afecta la **velocidad**, no qué opciones aparecen.
3. **La práctica cuenta.** Producir y usar algo también genera conocimiento (madurez).
4. **El conocimiento se puede adquirir de muchas formas** (ver fuentes): sirve a todos los estilos de juego (03).

---

## Cómo funciona la investigación

### Proyectos con laboratorios y científicos (decidido)

Estilo Aurora: se crea un **proyecto** por tecnología y se le asignan **laboratorios** y un **científico a cargo**.

```
PI_mes(lab) = 10 × (0,7 + 0,1 × habilidad_científico)      # habilidad de investigación 0–6
velocidad(proyecto) = Σ_labs PI_mes(lab) × 0,93^(n−1)     # n = orden del laboratorio en el proyecto
tiempo(proyecto) = costo_PI / velocidad
```

- **Rendimientos decrecientes:** cada laboratorio adicional en el mismo proyecto rinde algo menos (0,93^n); conviene repartir laboratorios entre proyectos.
- **Laboratorios:** son instalaciones (03, 07), estatales o privadas. Hay tipos por campo (un laboratorio de materiales no investiga sensores).
- **Científicos:** notables (06) con habilidad por campo, rasgos y lealtad. Un científico a cargo con buena habilidad en el **campo** del proyecto acelera; uno sin habilidad en ese campo penaliza.
- **Presupuesto:** los laboratorios y científicos cuestan mantenimiento. Subir el número de laboratorios requiere inversión y científicos formados (ADM-0.2).
- **Inicio por defecto (Tierra, 02):** ~10 laboratorios y ~15 científicos → **≈ 120 PI/mes**.

### Madurez (simplificada para el MVP)

| Estado | Efecto |
|---|---|
| **Teoría** | Se desbloquean los diseños con **menor confiabilidad** y costo mayor |
| **Madura** | Tras construir/usar un número de unidades (producción, misiones), baja el costo y mejoran confiabilidad (07) |

### Riesgos
Investigación peligrosa (reactores de fisión/fusión, hábitats, motores nucleares) tiene un **riesgo de accidente** baja probabilidad, proporcional a la inversión en seguridad; un accidente retrasa el proyecto y puede dañar instalaciones.

---

## Estructura: campos con niveles

La Fase 1 tiene **8 campos** y **4 niveles (eras 0–3)**. Cada tecnología tiene ID `CAMPO-era.número`. Un nivel no se bloquea estrictamente: se pueden tener tecnologías de varias eras si se cumplen los prerrequisitos.

| Código | Campo |
|---|---|
| **PRO** | Cohetería y propulsión |
| **ENE** | Energía |
| **MAT** | Materiales e industria |
| **SEN** | Sensores y astronomía |
| **COM** | Comunicaciones y computación |
| **VID** | Soporte vital y hábitats |
| **MIN** | Minería y procesamiento |
| **ADM** | Ciencias sociales y administración |

Campos de la Fase 2+ (se agregan después): armamento, seguridad y minas, biología y medicina avanzada, física (FTL propio, puntos de salto propios), ecología y terraformación, xenociencias, economía y finanzas.

### Eras de la Fase 1

| Era | Nombre | Hitos |
|---|---|---|
| **0** | Pre-orbital | Cohetes, satélites, vuelo tripulado orbital, agencia espacial |
| **1** | Órbita | Sondas, estaciones, telescopios orbitales, fisión compacta |
| **2** | Expansión local | Minería extraplanetaria, hábitats, relés, flotas y programas |
| **3** | Exploración profunda | Fusión, relevamiento gravimétrico, **sistemas de salto** |

### Escala de costos (puntos de investigación, PI)

| Era | Costo por tecnología | Cantidad | Total aproximado |
|---|---|---|---|
| 0 | 400–1.000 | 12 | 8.000 |
| 1 | 1.500–3.500 | 15 | 34.000 |
| 2 | 5.000–9.000 | 19 | 139.000 |
| 3 | 15.000–40.000 | 14 | 333.000 |

El **camino crítico hacia el FTL** exige sólo una parte (~55–60%) del total. La capacidad de investigación del Estado crece de ~120 a varios miles de PI/mes durante la fase. **Calibrar** con prototipos para que la Fase 1 dure entre 20 y 35 años de juego (02).

---

## Lista de tecnologías (60)

Tecnologías con estado **inicial (✔)**: SEN-0.1, COM-0.2, MAT-0.1 (la Tierra ya las tiene).

### Era 0 — Pre-orbital (12)

| ID | Tecnología | Prerrequisitos | Desbloquea | PI |
|---|---|---|---|---|
| PRO-0.1 | Cohetes químicos orbitales | — | Lanzador ligero; acceso a la órbita baja | 800 |
| PRO-0.2 | Lanzadores reutilizables | PRO-0.1, MAT-0.2 | −30% costo de lanzamiento | 1.000 |
| ENE-0.1 | Paneles solares espaciales | — | Energía para satélites y sondas | 500 |
| MAT-0.1 ✔ | Aleaciones y compuestos ligeros | — | Cascos ligeros | 600 |
| MAT-0.2 | Manufactura de precisión | MAT-0.1 | Componentes de calidad "buena" | 700 |
| SEN-0.1 ✔ | Telescopios terrestres mejorados | — | Estado **Sospecha** con rangos | 400 |
| SEN-0.2 | Radiotelescopios | SEN-0.1 | Pistas de cuerpos oscuros y señales | 700 |
| COM-0.1 | Satélites de comunicaciones | PRO-0.1, ENE-0.1 | Satélite de comunicaciones; base de la red | 600 |
| COM-0.2 ✔ | Computación avanzada | — | +5% a la velocidad de investigación | 800 |
| VID-0.1 | Vuelo tripulado orbital | PRO-0.1, MAT-0.1 | Cápsula tripulada | 1.000 |
| ADM-0.1 | Agencia espacial | — | Programa espacial; +capacidad administrativa | 400 |
| ADM-0.2 | Educación científica | — | Más científicos disponibles; laboratorios extra | 500 |

### Era 1 — Órbita (15)

| ID | Tecnología | Prerrequisitos | Desbloquea | PI |
|---|---|---|---|---|
| PRO-1.1 | Motores iónicos | PRO-0.1, ENE-0.1 | Sondas de larga duración (eficiencia ×5) | 2.500 |
| PRO-1.2 | Propulsión química criogénica | PRO-0.1, MAT-0.2 | Transferencias a Luna y planetas cercanos | 2.000 |
| ENE-1.1 | Reactores de fisión compactos | MAT-0.2, ENE-0.1 | Energía en órbita y bases | 3.500 |
| ENE-1.2 | Radiadores y gestión térmica | ENE-1.1 | Reactores más eficientes; calor como restricción | 1.800 |
| MAT-1.1 | Escudos térmicos y blindaje antimicrometeoritos | MAT-0.1 | Cascos resistentes | 1.800 |
| MAT-1.2 | Manufactura en órbita | MAT-0.2, VID-1.2 | Talleres orbitales | 3.500 |
| SEN-1.1 | Telescopios orbitales | SEN-0.1, PRO-0.1, COM-0.1 | Estado **Observado** en cuerpos cercanos | 2.500 |
| SEN-1.2 | Instrumental de sondas | SEN-0.2, COM-1.1 | Relevamiento por sobrevuelo | 2.500 |
| SEN-1.3 | Seguimiento de objetos cercanos | SEN-1.1 | Detección de asteroides y cometas; alerta de impactos (12) | 1.500 |
| COM-1.1 | Telemetría y control remoto | COM-0.1, COM-0.2 | Sondas operables a distancia | 1.500 |
| COM-1.2 | Xenolingüística y codificación de señales | COM-0.2, SEN-0.2 | Comunicación ×1,5 más rápida en primer contacto | 2.000 |
| VID-1.1 | Soporte vital orbital | VID-0.1 | Estancias largas en órbita | 2.500 |
| VID-1.2 | Estaciones modulares | VID-1.1, MAT-1.1 | Estación orbital con módulos | 3.000 |
| MIN-1.1 | Prospección con sondas | SEN-1.2 | Revela yacimientos en cuerpos sobrevolados | 1.800 |
| ADM-1.1 | Contratos con privados | ADM-0.1 | Empresas aeroespaciales privadas; servicios de lanzamiento | 1.500 |

### Era 2 — Expansión local (19)

| ID | Tecnología | Prerrequisitos | Desbloquea | PI |
|---|---|---|---|---|
| PRO-2.1 | Propulsión nuclear térmica | PRO-1.2, ENE-1.1 | Gran ΔV; transporte interplanetario | 9.000 |
| PRO-2.2 | Iónicos de alta potencia | PRO-1.1, ENE-1.2 | Naves de carga eficientes | 8.000 |
| ENE-2.1 | Fisión de alta potencia | ENE-1.1, MAT-1.1 | Colonias y estaciones grandes | 8.000 |
| MAT-2.1 | Astilleros orbitales | MAT-1.2, COM-2.2 | Construcción de naves grandes en órbita | 9.000 |
| MAT-2.2 | Aleaciones de alta resistencia | MAT-1.1, MAT-1.2 | Cascos y blindajes mejores | 7.000 |
| SEN-2.1 | Relevamiento geológico orbital | SEN-1.2, MIN-1.1 | Estado **Relevado** (yacimientos precisos) | 6.000 |
| SEN-2.2 | Sensores de nave activos y pasivos | SEN-1.1, COM-2.1 | Sensores para naves (11) | 8.000 |
| COM-2.1 | Relés interplanetarios | COM-1.1, ENE-1.1 | Latencia mitigable (01) | 7.000 |
| COM-2.2 | Automatización industrial | COM-0.2, MAT-1.2 | Menos mano de obra; mejor producción (07) | 8.000 |
| VID-2.1 | Hábitats superficiales presurizados | VID-1.2, MAT-2.2 | Bases y colonias en superficie | 9.000 |
| VID-2.2 | Medicina espacial y protección radiológica | VID-1.1 | Menos pérdida de salud; estancias largas | 6.000 |
| VID-2.3 | Agricultura y reciclaje cerrado | VID-1.1 | Colonias con alimento propio | 7.000 |
| MIN-2.1 | Minería robótica de superficie | MIN-1.1, COM-1.1 | Minas en Luna y planetas | 7.000 |
| MIN-2.2 | Minería de asteroides | MIN-1.1, PRO-1.2 | Extracción en asteroides | 9.000 |
| MIN-2.3 | Refinación in situ y combustible ISRU | MIN-2.1, ENE-1.1 | Combustible y metales procesados fuera del planeta | 8.000 |
| MIN-2.4 | Extracción de volátiles y agua | MIN-1.1, VID-1.1 | Agua y volátiles para soporte vital y combustible | 7.000 |
| ADM-2.1 | Logística y gestión de flotas | ADM-1.1, COM-1.1 | **Programas** (órdenes masivas) y contratos de servicios (07) | 6.000 |
| ADM-2.2 | Derecho espacial y propiedad de recursos | ADM-1.1 | Concesiones, licencias y reclamos; privados reclaman cuerpos | 5.000 |
| ADM-2.3 | Protocolos de primer contacto | COM-1.2, ADM-2.2 | Señal de paz, regalos, objetivo común (02, 10) | 5.000 |

### Era 3 — Exploración profunda (14)

| ID | Tecnología | Prerrequisitos | Desbloquea | PI |
|---|---|---|---|---|
| PRO-3.1 | Propulsión de fusión (primera generación) | PRO-2.1, ENE-3.1 | Naves rápidas de largo alcance | 30.000 |
| PRO-3.2 | **Sistemas de salto** | SEN-3.2, PRO-2.1, ENE-2.1 | **Cruzar puntos de salto (FTL)**; fin de la Fase 1 | 40.000 |
| ENE-3.1 | Fusión experimental | ENE-2.1, MAT-2.2 | Reactores de fusión de laboratorio | 25.000 |
| ENE-3.2 | Reactores de fusión compactos | ENE-3.1, MAT-3.1 | Energía para naves y estaciones grandes | 35.000 |
| MAT-3.1 | Superconductores y materiales avanzados | MAT-2.2 | Componentes avanzados | 25.000 |
| MAT-3.2 | Astilleros de gran escala | MAT-2.1, COM-3.1 | Naves y estaciones muy grandes | 20.000 |
| SEN-3.1 | Gravímetros de precisión | SEN-2.1, SEN-2.2 | **Relevamiento gravimétrico**: localizar puntos de salto | 20.000 |
| SEN-3.2 | Análisis de anomalías de puntos de salto | SEN-3.1 | Estudiar un punto de salto localizado | 30.000 |
| COM-3.1 | Control de flotas y logística asistida | COM-2.1, COM-2.2, ADM-2.1 | Asignación masiva y automática más eficiente | 18.000 |
| VID-3.1 | Colonias autosuficientes | VID-2.1, VID-2.3, MIN-2.3 | Colonias que no dependen del suministro externo | 20.000 |
| MIN-3.1 | Minería automatizada a gran escala | MIN-2.2, COM-2.2 | Minería con casi nula mano de obra | 18.000 |
| MIN-3.2 | Recolección de gases de planetas gigantes | MIN-2.3, PRO-2.1 | Hidrógeno y helio de gigantes gaseosos | 22.000 |
| ADM-3.1 | Gobierno de colonias distantes | ADM-2.1, VID-2.1 | +capacidad administrativa; gobernadores de sistema | 15.000 |
| ADM-3.2 | Diplomacia interestelar y embajadas | ADM-2.3, COM-1.2 | Embajadas permanentes; mejores acuerdos (10) | 15.000 |

**Condición de salida de la Fase 1:** PRO-3.2 (sistemas de salto) más un punto de salto localizado (SEN-3.1) y estudiado (SEN-3.2), y una nave capaz de cruzar.

---

## Fuentes de progreso

- **Investigación formal:** proyectos (arriba).
- **Aprendizaje por la práctica** (madurez, 07).
- **Datos de exploración:** muestras geológicas, anomalías y relevamiento dan bonificaciones a campos concretos (SEN, MIN, VID).
- **Ingeniería inversa** de ruinas o naves capturadas: con muestras y un laboratorio, reduce **30–60%** el costo en PI de tecnologías relacionadas (según estado de las muestras y del laboratorio).
- **Intercambio y espionaje** (decidido): comprar o licenciar tecnología a otras facciones (10); **robar** planos o datos con operaciones de espionaje (20). Una tecnología robada o licenciada reduce el costo, no lo elimina, y puede exigir que se cumpla algún prerrequisito.
- **Empresas privadas:** a partir de ADM-1.1 las empresas aeroespaciales desarrollan su **propia tecnología civil** (lanzadores, satélites, naves de carga, minería). El jugador puede:
  - **comprar** sus naves (07),
  - **alquilar servicios** (fletes, minería, colonización, relevamiento) por contrato (07),
  - **licenciar o comprar** su tecnología.
  ```
  precio_licencia ≈ 150 cr × costo_PI × (1 − descuento_relación)    # valor inicial a calibrar
  ```
  Las empresas **no investigan tecnología militar** salvo en regímenes corporativos (03).
- **Conquista, inmigración de científicos y ruinas precursoras** (imperialista, 03; 14).

## Difusión y pérdida

- Las tecnologías se difunden por comercio y contacto.
- Políticas de secreto, patentes, protección de planos.
- **Pérdida:** si mueren todos los que la dominaban y se destruyen los archivos, se pierde. Posibles **edades oscuras**. La tecnología precursora es conocimiento perdido.

## Tecnología precursora

- Artefactos parcialmente comprensibles; a veces peligrosos.
- Pueden dar **saltos** tecnológicos (reducen costos de varias tecnologías o dan acceso directo).
- No siempre reproducibles.

---

## FTL: puntos de salto (decidido)

### Puntos de salto naturales
- **Red fija** de conexiones entre sistemas, a **descubrir** mediante relevamiento gravimétrico (SEN-3.1).
- Algunos puntos de salto son **ocultos** (se revelan con sensores/relevamiento avanzado).
- Son **cuellos de botella** estratégicos: defensa, minas, patrullas, bloqueos (11).

### Puntos de salto propios (decidido: inversión media, escalable)
- El jugador puede **construir** puntos de salto, reconfigurando la red. **Accesibles en la Fase 2**: requieren tecnología avanzada (no tardía), materiales y energía; módulo/nave de construcción (07).
- **Se pueden tener varios**, pero **cada uno suma costo y puntos débiles**: mantenimiento logístico, nuevas rutas que defender, nuevos accesos para el enemigo.
- **Mantenimiento logístico obligatorio:** recursos, energía, personal y reposición periódica; sin él se **degrada** y puede cerrarse o volverse inestable (12).
- **Dónde ponerlos es una decisión estratégica.** Otras civilizaciones también pueden abrir puntos de salto, incluso hacia lugares desconocidos para vos (incursiones, 11, 14).
- *Costos exactos, alcance y escalado: a definir con prototipos.*

## Comunicaciones y relés

Los **relés de comunicaciones** (COM-2.1) reducen la latencia de información entre sistemas (01); los avanzados (FTL) también **aumentan la capacidad administrativa** (02): tecnología llave para la Fase 3.

## Conexiones

01 (información y relés) · 02 (eras y condiciones de salida) · 03 (empresas de I+D) · 06 (científicos) · 07 (componentes, diseños de fábrica, programas y contratos) · 09 (espionaje) · 10 (intercambio, licencias) · 11 (sensores, minas) · 12 (fallas, impactos) · 14 (ruinas, anomalías) · 20 (robo de planos).

## Preguntas abiertas

- Calibrar costos, ritmo de PI y la duración de la Fase 1.
- Lista de tecnologías de la Fase 2+ (armamento, seguridad, física, etc.).
- ¿Se ve la red completa de puntos de salto o sólo la parte descubierta? (Propuesta: sólo lo descubierto.)
- Costos, alcance y escalado de los puntos de salto propios.
- Definir la operación de espionaje "robo de tecnología" en 20 (probabilidades y reducción de costo).
- Cantidad y tipo de laboratorios (generales vs. por campo) y costo de mantenimiento.
