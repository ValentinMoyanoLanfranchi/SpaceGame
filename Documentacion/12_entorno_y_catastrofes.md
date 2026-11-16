# 12 — Entorno, clima y catástrofes

> v0.3 · Aclara el alcance en el MVP: la Tierra es un único nodo (05), por lo que las catástrofes planetarias la afectan **como un todo** (con una **magnitud**), y en el MVP se simulan sólo unas pocas (ver "Alcance en el MVP"). Las regiones abstractas se usan en planetas y lunas grandes después del MVP.

## Modelo planetario

Parámetros por cuerpo celeste:
- Masa / gravedad, radio.
- **Atmósfera:** composición, presión.
- **Temperatura** (según distancia a la estrella, atmósfera y región).
- Hidrosfera (océanos, hielo).
- Biosfera (de inexistente a rica).
- **Magnetosfera** (protección contra radiación).
- Actividad tectónica y volcánica.
- Duración del día y del año; inclinación axial; excentricidad orbital (circular en el MVP, 21).
- Mareas (por lunas cercanas).
- Luminosidad y actividad de la estrella.

Los planetas y lunas grandes **habitados** se dividen en **regiones abstractas** (sin mapa de superficie) con bioma/entorno propio (05, 14). **Excepción en el MVP: la Tierra es un único nodo**, con parámetros agregados (clima global, tectónica global); los cuerpos pequeños (Luna, asteroides, Marte inicial) también son nodos únicos. El clima se simplifica a **parámetros por región o por nodo** (temperatura, precipitaciones, estaciones) y **eventos** estadísticos.

## Habitabilidad

Por especie: puntaje de compatibilidad **por región (o por nodo)** y requisitos de infraestructura (desde "vivir al aire libre" hasta "cúpula presurizada con blindaje antirradiación"). La terraformación cambia parámetros lentamente.

---

## Catástrofes

### Naturales planetarias (afectan regiones o, en un nodo único, a todo el nodo)

| Catástrofe | Escala | Precursores detectables | Efectos típicos |
|---|---|---|---|
| Terremoto | Regional | Sismología | Daño a instalaciones e infraestructura |
| Erupción volcánica / supervolcán | Regional → planetaria | Monitoreo volcánico | Ceniza, invierno volcánico, hambrunas |
| Huracanes / tormentas | Regional | Meteorología | Daños, producción y transporte interrumpidos |
| Inundaciones / sequías | Regional | Clima | Cosechas perdidas |
| Olas de calor / frío | Regional | Meteorología | Mortalidad, demanda de energía |
| Tormentas de polvo (mundos áridos) | Planetaria | Meteorología | Energía solar inutilizada por semanas |
| Impactos de asteroides | Regional → extinción | Vigilancia orbital | Desde cráteres hasta colapso global |
| Plagas agrícolas | Regional | Biología | Hambrunas |
| Infestaciones de fauna | Regional | Observación | Cultivos perdidos, ataques |
| **Pandemias** | Variable | Medicina | Muertes, cuarentenas, colapso económico; contagios entre especies |
| Eras glaciales | Planetaria (lento) | Climatología | Cambio de habitabilidad |

### Espaciales

| Catástrofe | Efectos |
|---|---|
| Fulguraciones / eyecciones de masa coronal | Radiación, electrónica y sensores dañados, redes caídas |
| Tormentas de radiación | Daño a tripulaciones y colonias sin blindaje |
| Micrometeoritos | Brechas en hábitats y naves |
| **Síndrome de Kessler** | Cascada de escombros que puede **cerrar la órbita** de un planeta por años (tema clave si los lanzamientos y satélites se multiplican en la Fase 1) |
| Variabilidad estelar | Cambios en energía disponible y clima |
| Anomalías de puntos de salto | Puntos de salto que se desestabilizan, se cierran o se abren (14) |
| Supernova cercana (rarísima) | Evento de sector |
| Anomalías | Fenómenos desconocidos (contenido de exploración) |

### Antropogénicas

Contaminación · colapso ecológico · sobreexplotación de suelos y acuíferos · accidentes industriales · fusión de reactores · brechas de hábitat · fallas en cascada de soporte vital · fuga de armas biológicas · terraformación fallida · especies invasoras traídas de otros mundos · **fallas de mantenimiento en puntos de salto propios** (13). En la Fase 1 se agregan los **accidentes de lanzamiento y de misión** (fallos de cohete, pérdida de sondas, accidentes de laboratorio por investigación peligrosa, 13).

---

## Modelo de ocurrencia

- **Probabilidades derivadas de parámetros físicos:** un planeta tectónicamente activo tiene más terremotos; la estrella determina las fulguraciones; la actividad industrial, la contaminación.
- **Nunca dirigidas al jugador** (ver director en 01).
- **Predicción:** con tecnología (sismógrafos, observatorios solares, vigilancia de asteroides, telescopios SEN-*) se gana **tiempo de aviso**.

## Alcance en el MVP

Para no inflar el MVP (21), sólo se simulan:
- **Fallos de lanzamiento y de misión** (probabilidad según madurez de la tecnología, 13, 07).
- **Accidentes de investigación peligrosa** (reactores, motores nucleares, hábitats).
- **Fulguraciones solares** que afectan satélites, sondas y colonias sin blindaje.
- **Un evento planetario simple en la Tierra** (p. ej. crisis climática, pandemia o sequía) con **magnitud global**, como presión sobre el apoyo público y la economía (05, 09).

El resto del catálogo queda para después del MVP.

## Cascadas (ejemplo)

```
Fulguración solar
 └► falla de la red eléctrica en Estación Beta
     └► soporte vital en baterías (72 h)
         └► racionamiento de oxígeno
             └► disturbios
                 └► el gobernador pide ayuda
                     └► facción vecina ofrece auxilio… a cambio de una concesión minera
```

Las catástrofes **alimentan todos los demás sistemas**. En la Fase 1, un ejemplo de cascada: *falla de lanzamiento con víctimas → caída del apoyo público → recorte del presupuesto espacial → proyectos demorados → empresas privadas aprovechan el vacío*.

## Respuesta

- **Preparación:** reservas, refugios, códigos de construcción, redundancia, alerta temprana.
- **Servicios de emergencia** (05).
- **Ayuda humanitaria** (diplomacia, 10).
- Seguros (03), reconstrucción.
- **Legitimidad:** una buena respuesta la sube; una mala puede tumbar un gobierno.

---

## Ecología

- Biosfera simplificada por región (o por nodo): poblaciones de fauna y flora, cadenas tróficas abstractas.
- Sobreexplotación y deforestación tienen efectos.
- **Especies invasoras** interplanetarias como catástrofe ecológica.

## Terraformación

- Proyectos de décadas o siglos que modifican parámetros (atmósfera, temperatura, agua).
- Conflictos: lo que vuelve habitable un planeta para unos puede volverlo inhabitable para otros.
- Fuera del MVP (tecnologías de la Fase 2+, 13).

## Conexiones

01 (director) · 03 (shocks, seguros) · 04 (rutas cortadas) · 05 (malestar, migración, apoyo público) · 07 (fallos de misión, madurez) · 08 (habitabilidad por especie) · 10 (ayuda humanitaria) · 13 (predicción, terraformación, riesgos de investigación) · 14 (generación de planetas) · 21 (alcance del MVP).

## Preguntas abiertas

- Profundidad del modelo climático por región (¿valores estáticos con eventos o simulación simple?).
- ¿Las pandemias son un sistema propio (vectores, mutaciones)?
- Frecuencia por defecto de catástrofes mayores (configurable por el jugador).
- Probabilidades base de los fallos de lanzamiento y de misión (calibrar con 07, 22).
