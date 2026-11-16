# 03 — Economía

> v0.3 · Alineado con 18 (bienes y parámetros), 19 (impuestos, incentivos, mercados grises, leyes laborales), 07 (empresas aeroespaciales privadas, programas) y 22 (datos del MVP). Este documento define las reglas; los números están en 18, 19 y 22.

## Objetivo

Una economía **emergente y simulada por agentes**, con precios locales, flujos físicos de bienes, empresas estatales y privadas, comercio entre facciones y crisis que surgen solas. El jugador gobierna un Estado que puede **dirigir la economía, dejarla al mercado, o combinar ambas**, y puede jugar de formas muy distintas (ver "Estilos de juego").

---

## Actores económicos

| Actor | Rol |
|---|---|
| **Pops** | Consumen según clase y necesidades; trabajan por salario; ahorran; pagan impuestos |
| **Empresas estatales** | Controladas por el jugador (o por el Estado, en la IA). Típicas: extracción de recursos necesarios, refinerías de combustible, minas, investigación y tecnología, educación, astilleros, transporte estratégico. **Modelo híbrido:** el jugador puede gestionar instalaciones individualmente o agruparlas en una empresa con director, presupuesto y directivas (18) |
| **Empresas privadas** | Poseen instalaciones y naves, contratan, producen, venden, invierten y quiebran. Pueden entrar en **los mismos rubros que el Estado** y en otros (bienes de consumo, comercio, servicios), salvo en los **rubros reservados**. Pueden crecer hasta ser megacorporaciones |
| **Empresas aeroespaciales privadas** | Aparecen cuando existe la tecnología base (ADM-1.1). Desarrollan sus propios diseños de lanzadores y naves: el Estado puede **comprar naves, alquilar servicios (lanzamientos, transporte, relevamiento) y comprar o licenciar tecnología** (07, 13) |
| **Comerciantes y transportistas** | Empresas privadas de transporte (cargueros civiles) que compran barato y venden caro (estilo X4) |
| **Bancos** | Crédito a empresas y Estados con interés según riesgo |
| **Estado** | Cobra impuestos, compra, subsidia, regula, posee empresas, comercia con otros Estados |
| **Facciones extranjeras** | Sus empresas (privadas o estatales) comercian en tus mercados si hay pactos |

> **Estados totalmente estatales:** algunas civilizaciones (p. ej. regímenes autoritarios o colectivistas) **no tienen empresas privadas**: todo es estatal y el comercio se hace Estado–Estado. Las especies y regímenes determinan qué actores existen (08, 09, 20).

---

## Bienes

El núcleo tiene **~73 bienes** (detalle y IDs en 18), extensibles por mods. **El MVP usa un subconjunto de 16** (22).

**Categorías:** crudos metálicos y minerales · volátiles y orgánicos · exóticos · refinados · componentes · bienes de consumo · bienes de capital · militares · información (datos de relevamiento, planos, licencias) · servicios y energía (locales).

**Propiedades de cada bien:** masa, volumen, perecibilidad, peligrosidad, legalidad por facción, preferencias culturales (un lujo para una cultura puede ser tabú para otra). Los minerales son **reales** con unos pocos exóticos ficticios (los "cristales de presión" del MVP, 22).

---

## Mercados

### Nodos (decidido: precios por nodo)
Cada colonia o estación es un **nodo de mercado** con stock, demanda y precio propio según stock vs. objetivo. Los nodos se vinculan por rutas (dentro del sistema o por puntos de salto) con costo, tiempo y riesgo de transporte. **En el MVP la Tierra es un único nodo** (05, 22); Luna y Marte son nodos propios.

### Formación de precios (modelo base)

```
precio = precio_base × clamp( (stock_objetivo / max(stock_actual, ε)) ^ elasticidad, 0.25, 6.0 )
```

- `stock_objetivo` = consumo esperado × días de cobertura (por grupo de bienes, ver 18).
- Elasticidad según el bien (comida y combustible: alta urgencia; lujo: baja).
- Suavizado temporal para evitar oscilaciones.
- Cadencia: los mercados se actualizan **semanalmente**; empresas y sociedad, mensualmente (01, 21).

### Órdenes y arbitraje
Las instalaciones publican **órdenes de compra y venta**. Los transportistas leen y arman viajes comparando `precio_destino − precio_origen − costo_transporte − riesgo`. Así los precios convergen a lo largo de rutas seguras y **divergen** cuando una ruta se corta o es insegura: un bloqueo o una zona de piratería se ven en los precios.

### El costo de lanzamiento (cuello de botella de la Fase 1)
Subir carga desde la superficie domina la economía espacial inicial: en los datos del MVP, un lanzador LV-1 pone 2 t en órbita por ~30 M cr y un LV-2 pone 15 t por ~70 M cr (22). Consecuencias: al principio **sólo conviene subir bienes de alto valor por masa** (instrumentos, electrónica, personas); la industria orbital y la minería en cuerpos de baja gravedad se vuelven rentables al reducirse ese costo; el presupuesto espacial (fracción del ingreso fiscal ligada al apoyo público, 22) fija el techo de lo que se puede lanzar.

---

## Relación Estado – sector privado

El jugador **regula, compra, grava y compite** con el sector privado.

### Entrada de privados (decidido)
Los privados **entran por rentabilidad** (invierten donde esperan retorno suficiente y tienen capital), con límites que define el jugador:
- **Rubros reservados** al Estado (p. ej. defensa, combustible estratégico).
- **Licencias** obligatorias para ciertos cuerpos, recursos o sistemas (con tarifa y/o **regalías**).
- **Aranceles** y otras regulaciones.
- **Incentivos** (19): exenciones, créditos fiscales y subsidios para atraer inversión a rubros o regiones concretas.

### Herramientas

| Herramienta | Efecto |
|---|---|
| **Rubros reservados / abiertos** | Define qué actividades son monopolio estatal y cuáles admiten privados |
| **Licencias y concesiones** | Autorizan extracción o comercio en ciertos cuerpos o sistemas; permiten cobrar |
| **Compra estatal de producción privada** | El Estado compra recursos a empresas privadas (contratos, cuotas, precios de sostén) en lugar de producirlos |
| **Programas y órdenes masivas** | Encargar de una vez muchas unidades a astilleros privados o estatales, o pagar a civiles por tareas repetitivas, sin microgestión (07) |
| **Importación estatal** | El Estado compra en otro sistema o planeta y lo transporta al lugar donde se necesita, con su flota o mediante contratos de flete |
| **Impuestos por categoría** | Renta, consumo, empresas, regalías de extracción, aranceles, por cabeza; tasas distintas por categoría de bien y de actividad (19) |
| **Incentivos y subsidios** | Exenciones, créditos y subsidios por sector o región (19) |
| **Leyes laborales** | Jornada, salario mínimo, sindicatos, servidumbre: afectan productividad, costos y malestar (19, 09) |
| **Controles de precios, racionamiento, embargos** | Generan escasez y **mercado gris** si se abusa (19) |
| **Nacionalización / privatización** | Cambia la propiedad de instalaciones; tiene costo político (09) |
| **Reservas estratégicas** | El Estado acumula bienes críticos (combustible, minerales) para crisis |

### Crecimiento y recaudación
Mientras **más se comercia**, más crece la economía y más recauda el Estado:

```
actividad_comercial(nodo) = Σ valor de transacciones del período
recaudación = Σ (tasa × base imponible) − evasión − corrupción
```

Esto crea un ciclo virtuoso (comercio → riqueza → recaudación → infraestructura y seguridad → más comercio), con **límites**: congestión de rutas, desigualdad, dependencia de importaciones, vulnerabilidad a bloqueos, y poder político de las grandes corporaciones (09).

---

## Comercio exterior y pactos

- **Pactos comerciales** con otras especies/facciones: habilitan que **las empresas de ambos lados comercien** (aranceles, cuotas, puertos autorizados).
- Con Estados totalmente estatales, el comercio es **Estado–Estado**: contratos, trueques de bienes y tecnología, tributos.
- **Embargos y sanciones** cortan el comercio por razones políticas.
- **Acuerdos de servicios** (ver 10): el jugador puede **pagar** a otra potencia por protección, recursos, desarrollo o tecnología, o ofrecerlos. La **licencia de tecnología** es una fuente de investigación (13).
- **Inversión extranjera:** empresas foráneas pueden invertir en tu territorio (y al revés), con riesgos de dependencia.

---

## Estilos de juego económico

| Estilo | Cómo funciona | Ventajas | Riesgos |
|---|---|---|---|
| **Todo estatal** | Empresas estatales en todo; privados prohibidos o mínimos | Control total, planificación, prioridades militares | Alto costo administrativo, ineficiencia, menos crecimiento por comercio |
| **Capitalismo puro** | Estado mínimo; privados hacen casi todo | Crecimiento por volumen de comercio, bajo costo admin | Poco control, dependencia de corporaciones, desigualdad |
| **Mixto** | Estatal en lo estratégico, privado en el resto | Equilibrio | Complejidad de gestión |
| **Imperialista** | Conquistar para obtener recursos, tecnología y conocimiento | Acceso a lo que no tenés | Guerra, rebeliones, cansancio, reputación (11, 10) |
| **Cliente / protectorado** | Pagar por protección, recursos, tecnología o desarrollo | Crecimiento acelerado, seguridad prestada | Dependencia, tributo, pérdida de autonomía (10) |
| **Autárquico** | Hacerlo todo desde cero | Independencia | Lento; falta de recursos clave |

Estos estilos no son modos cerrados: son combinaciones de régimen (09, 20), leyes económicas y decisiones del jugador.

---

## Dinero y moneda

- Monedas de facción con **tipos de cambio** flotantes según balanza comercial y **confianza en el emisor**. Una facción que colapsa deja su moneda sin valor.
- **Emisión:** imprimir dinero financia gastos pero genera inflación.
- **Respaldo:** monedas respaldadas por metal, por energía, o fiduciarias.
- Posible **moneda interestelar** de un bloque o de una hegemonía.
- Los Estados totalmente estatales pueden operar con un sistema de cuentas/asignación en vez de una moneda comercial.
- **MVP:** una sola moneda (crédito, cr); tipo de cambio y emisión se difieren (21).

---

## Trabajo

- Los trabajos requieren habilidades/educación (educación = empresa estatal o privada); salarios por oferta y demanda local.
- **Desempleo** → malestar y emigración. **Escasez de mano de obra** → salarios ↑, automatización, inmigración.
- La automatización (07) desplaza trabajadores → tensión política.
- **Leyes laborales y servidumbre** (19, 09): la servidumbre es opcional, sube la producción pero provoca revueltas, sanciones y daño de reputación.

---

## Finanzas

- **Crédito:** bancos prestan con interés según riesgo. Los Estados emiten bonos.
- **Default:** daño de reputación, embargos, pérdida de activos, intervención extranjera.
- **Inversión:** las empresas privadas construyen cuando esperan rentabilidad. El jugador puede invertir en empresas ajenas y, más tarde, mercados de acciones.
- **Seguros:** contra piratería y catástrofes; las **primas por ruta/sistema** son un indicador útil de seguridad (11).

---

## Mercados negros, grises y contrabando

Cada facción define bienes ilegales, y las **regulaciones excesivas** (controles de precios, racionamiento, aranceles prohibitivos, impuestos altos) generan **mercados grises**: parte de la actividad pasa a la informalidad, con evasión y corrupción de aduanas (19). Surgen rutas de contrabando, precios con prima, sindicatos criminales y, en la Fase 2, piratas que viven de ello (09, 11).

---

## Ciclos y crisis (emergentes)

- **Shock de oferta:** se pierde una mina o una ruta → precio sube → especulación → sobreinversión → exceso de oferta → quiebras.
- **Bloqueo de un punto de salto:** colonias dependientes se quedan sin suministros.
- **Corrida bancaria** tras rumores de default.
- **Hiperinflación** por financiar una guerra imprimiendo dinero.
- **Economía de guerra:** conversión industrial, racionamiento, posguerra con desempleo.
- **Quiebra de una megacorporación** con impacto político.

Variable auxiliar: **"sentimiento de mercado"** por nodo, influido por noticias y rumores (01, 15).

---

## Economía por fase

| Fase | Características |
|---|---|
| 1 — Sistema local | Mercado planetario único (Tierra) más nodos orbitales y colonias; empresas estatales y privadas (incluidas aeroespaciales); el costo de lanzamiento limita lo que se puede subir; industria orbital y minería espacial como salida |
| 2 — Interestelar | Comercio entre sistemas y facciones; pactos; rutas con riesgo; los precios lejanos están desactualizados por la latencia de información (01): especulación con información |
| 3 — Sector | Macroeconomía, bloques comerciales, monedas hegemónicas, megacorporaciones |

## Herramientas para el jugador

- Gráficos de flujo de bienes, historial de precios, mapas de calor por sistema.
- **"¿Por qué cambió este precio?"** con desglose (stock, rutas cortadas, demanda nueva, inseguridad).
- Proyecciones simples ("a este ritmo, sin combustible en 12 días").
- Panel del **sector estatal vs. privado** (participación, rentabilidad, recaudación).

## Indicadores clave

PBI por nodo/facción, volumen de comercio, inflación, desempleo, balanza comercial, deuda, índice de precios de canasta básica por clase, participación estatal, presupuesto espacial disponible.

## Conexiones

04 (transporte y stocks físicos) · 05 (consumo y trabajo) · 07 (producción, programas, empresas aeroespaciales) · 09 (régimen, grupos de interés) · 10 (pactos, embargos, acuerdos de servicios) · 11 (economía de guerra, seguridad) · 12 (shocks) · 13 (licencias de tecnología) · 18 (bienes y parámetros) · 19 (impuestos, incentivos, mercados grises) · 20 (regímenes) · 22 (datos del MVP).

## Preguntas abiertas

- ¿Cuántas empresas privadas se simulan individualmente vs. agregadas por sector? (MVP: 3–4 empresas individuales, 22.)
- ¿Mercado de acciones sí o no? (Fuera del MVP.)
- ¿Cómo se balancea el costo administrativo de las empresas estatales frente al crecimiento privado?
- Calibración de precios, ratios y márgenes (18) y del costo de lanzamiento frente al presupuesto espacial inicial (~600 M cr/año, 22).
