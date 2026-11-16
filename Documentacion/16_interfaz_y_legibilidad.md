# 16 — Interfaz y legibilidad

> v0.3 · Agrega las pantallas del MVP, la UI de programas y órdenes masivas, los estados de conocimiento, la relación visible con tendencia, los objetivos sugeridos y la plataforma (PC con Godot 4, 21).

## Principio

Pilar 7: **profundo pero legible e informado.** Un juego así vive o muere por sus herramientas de información. El jugador tiene que poder entender qué pasa y por qué, y **nunca quedarse sin información ni opciones** en una crisis (11).

## Estilo visual y plataforma (decidido)

**2D.** Mapas cenitales limpios de sistemas y galaxia. Prioridad: legibilidad y rendimiento sobre espectacularidad. **Plataforma objetivo: PC** (Windows, Linux, macOS) con **Godot 4 y gdext** sobre el núcleo de simulación en Rust (21); no se contempla móvil.

## Vistas por escala

Zoom semicontinuo: **colonia → planeta → sistema → sector → galaxia**. Cada nivel muestra la información apropiada.

- **Mapa de sistema:** órbitas en rieles, flotas, contactos, sensores, rutas, puntos de salto, **estado de conocimiento de cada cuerpo**.
- **Mapa de galaxia:** grafo de sistemas y puntos de salto, facciones, rutas comerciales, seguridad (Fase 2).
- **Vista de planeta/colonia:** instalaciones, población, infraestructura, malestar; regiones sólo en planetas grandes después del MVP (05).

## Inspección causal

- Botón / tooltip **"¿Por qué?"** en cualquier valor: muestra el árbol de modificadores.
- Ejemplos: "¿Por qué hay malestar en la Luna?" → desempleo (+10), impuestos (+15), escasez de agua (+12). "¿Por qué subió el precio del combustible?" → stock bajo, costo de lanzamiento, riesgo de ruta.
- **"¿Por qué nos odian/aprecian?"** en la relación con otra facción (ver abajo).
- Enlaces a eventos causales del registro (15).

## Paneles principales

Ventanas acoplables estilo Aurora, con plantillas de disposición.

| Panel | Contenido |
|---|---|
| Economía | Flujos, precios, producción, consumo, balanzas; **sector estatal vs. privado**, recaudación, presupuesto espacial |
| Empresas | Empresas estatales (directas) y privadas (incluidas aeroespaciales): indicadores, licencias, contratos, **catálogo de servicios y naves en venta** |
| **Programas** | Programas y órdenes masivas (ver abajo), estado y costo |
| Logística | Red, rutas, cuellos de botella, alertas de abastecimiento |
| Población | Pops, felicidad y lealtad, migración; **apoyo público** y grupos de interés (Tierra) |
| Política | Grupos de interés, leyes, legitimidad, corrupción, servidumbre, impuestos e incentivos |
| Diplomacia | Matriz de relaciones **con tendencia**, tratados, acuerdos de servicios, reputación |
| Militar | Flotas, **doctrinas**, suministro, inteligencia (Fase 2) |
| **Seguridad** | Niveles por sistema/zona, patrullas, puestos de vigilancia, minas, sensores (Fase 2) |
| **Investigación** | Laboratorios y científicos asignados, PI/mes, proyectos en curso, red tecnológica (8 campos × 4 eras), fuentes de avance (13) |
| **Conocimiento del sistema** | Lista de cuerpos con su estado (Desconocido → Sospecha → Observado → Relevado → Explorado), zonas de relevamiento gravimétrico |
| Diseño | Naves, unidades, instalaciones, puestos; **diseños de fábrica** y su comparación con el diseño propio (07) |
| Notables | Gobernadores, almirantes, directores, científicos, líder: lealtad y habilidades; ficha de líder (20) |
| Crónica | Historia, figuras, eventos |

## Programas y órdenes masivas (anti-microgestión)

Para evitar el micromanejo con muchas naves, instalaciones y civiles (07):

- **Programa:** orden única con objetivo y presupuesto (p. ej. "mantener 4 cargueros Tierra–Luna", "relevar todos los asteroides del cinturón", "producir 10 sondas").
- **Orden masiva:** asignar muchas naves o pagar a empresas civiles por una tarea repetitiva en una sola acción.
- Cada programa muestra **estado, cumplimiento, costo mensual y qué lo bloquea**, con el "¿por qué?" habitual.
- Se habilitan con la tecnología ADM-2.1 (13).

## Relación visible (diplomacia)

Cada facción contactada muestra un **indicador de relación** con **tendencia** (sube / estable / baja) y un **desglose** de factores (especie/cultura, régimen, líder, agravios, intereses, regalos, objetivo común). El botón "¿por qué nos odian/aprecian?" abre el desglose (10). Se evita que el jugador no sepa cuándo mejoran o empeoran las relaciones.

## Estados de conocimiento y fichas de líderes

- Los cuerpos y puntos de salto se muestran con su **estado de conocimiento**: Desconocido, Sospecha (datos con rangos amplios), Observado, Relevado, Explorado (02).
- Las fichas de líderes extranjeros muestran lo que el jugador **sabe** (niveles N0–N3, 20); los campos desconocidos aparecen como "?" o como rango.

## Objetivos sugeridos y ayudas

Lista **opcional** de objetivos sugeridos (primer satélite, primera sonda, estación, relevar la Luna, primera colonia, minería de asteroides, localizar y cruzar un punto de salto; 02), con **ayudas contextuales**. No hay misiones obligatorias ni tutorial forzado.

## Pantallas del MVP

El MVP (21) incluye un conjunto mínimo de pantallas: **mapa de sistema** (con estados de conocimiento), **panel de investigación**, **panel de economía y presupuesto**, **panel de empresas y programas**, **diseño de naves** con diseños de fábrica, **panel de notables y líder**, **registro de eventos y alertas**, **control de tiempo**. Diplomacia, militar y seguridad llegan después (Fase 2).

## Superposiciones de mapa

Recursos · malestar · rutas y flujos · control territorial · **nivel de seguridad** · cobertura de sensores · **antigüedad de la información** · **estado de conocimiento** · clima · contaminación · rutas de patrulla.

## Visualizar la incertidumbre

- Edad de los datos ("hace 12 días").
- Rangos en vez de números exactos (flota enemiga: 8–14 naves; masa de un cuerpo en estado Sospecha).
- Datos "fantasma" (última posición conocida de una flota).
- Calidad de contacto (desconocido → tipo → clase → composición).

## Alertas, ventanas de decisión y tiempo

- **Alertas configurables** por tipo y prioridad; agrupación para evitar spam.
- **Alerta de contacto:** origen probable, hora estimada de llegada, composición estimada, nivel de amenaza (11, Fase 2).
- **Alerta de anomalía:** señal detectada en una zona con posibles interpretaciones (incursión por punto de salto desconocido, fenómeno natural, etc.).
- **Ventana de decisión:** el juego se ralentiza solo ante un contacto hostil y muestra opciones claras (huir / resistir / pedir refuerzos / cambiar doctrina / dejar actuar a la doctrina), con probabilidades y ETA.
- **Auto-pausa** configurable por tipo de evento (en el MVP: tecnología completada, lanzamiento fallido, relevamiento completo, apoyo público bajo).
- **Informes de combate** con resumen y "por qué": quién disparó, qué falló, qué decisión se tomó.
- Controles de tiempo por incrementos (5 s · 1 h · 1 día · 5 días · 30 días).

## Automatización y plantillas

- Doctrinas y ROE estándar por plantilla.
- Perfiles de gobernador (directivas predefinidas).
- Plantillas de diseño (naves, unidades, instalaciones, puestos) y diseños de fábrica.
- Rutas logísticas guardadas y órdenes de compra estatal.
- Rutas de patrulla guardadas.
- Programas guardados.

## Enseñanza progresiva

- Se empieza con **un solo planeta** (la Tierra) y pocos sistemas activos.
- Cada fase y cada estado de conocimiento **revela** contenido gradualmente (*progressive disclosure*).
- Enciclopedia interna con enlaces.
- Modo tutorial guiado opcional para economía mixta y combate dinámico.

## Accesibilidad

Escalado de interfaz, modos para daltonismo, atajos configurables, velocidad de juego libre.

## Moddeabilidad

Diseñar desde el inicio con **datos externos** (especies, bienes, recetas, tecnologías, eventos, culturas, componentes, doctrinas, cuerpos celestes) en archivos JSON editables (21, 22).

## Conexiones

Todos los documentos. En particular: 02 (objetivos, conocimiento), 07 (programas, diseños de fábrica), 10 (relación visible), 13 (investigación), 20 (fichas de líderes), 21 (plataforma y pantallas del MVP), 22 (datos).

## Preguntas abiertas

- ¿Cuántos paneles visibles a la vez? (Ventanas acoplables estilo Aurora vs. interfaz más guiada.)
- Nivel de personalización de alertas y ventanas de decisión.
- Diseño concreto del panel de programas y del panel de investigación (prototipar en M3–M5, 21).
