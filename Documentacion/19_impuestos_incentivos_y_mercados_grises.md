# 19 — Impuestos por categoría, incentivos y mercados grises

> **Estado:** borrador v0.1. Valores orientativos, a calibrar con el prototipo de economía (Etapa B). Complementa a `03_economia.md`, `09_facciones_y_politica_interna.md`, `10_diplomacia.md` y `18_bienes_cadenas_y_parametros_economicos.md`.

## Objetivo

Dar al jugador **palancas económicas finas** para dirigir el desarrollo sin controlar cada instalación: bajar impuestos a un sector para atraer empresas, comprar más barato lo que necesita (metales para naves, satélites, estaciones, armas), invertir en logística, y también **jugar en la zona gris**: sustancias reguladas o prohibidas, contrabando y leyes de trabajo duras, siempre con **ventajas y desventajas reales**.

---

## 1. Impuestos por categoría (decidido: global + categoría + bien, con herencia)

### Jerarquía

```
Global → Grupo (metales, alimentos, componentes…) → Sector (bebidas, tecnología de consumo…) → Bien
```

- Cada nivel hereda del superior salvo que tenga un **override**. **El más específico manda.**
- La UI muestra un árbol con "heredado / modificado" (16).
- Aplica a varios instrumentos, cada uno con su propia jerarquía:

| Instrumento | Quién paga | Ejemplo |
|---|---|---|
| **Consumo** | Compradores | Menos impuesto a alimentos básicos |
| **Selectivos** | Consumidores de bienes específicos | Alto a licores y tabaco |
| **Regalías** | Extractores | Menos a hierro, más a platinoides |
| **Empresas (por rubro)** | Productores | Menos a metalurgia |
| **Aranceles** | Importadores/exportadores | Alto a lujo importado, nulo a insumos críticos |

### Restricciones
- Los regímenes con contrapesos (09) **limitan la velocidad** de cambio de tasas: en una democracia, modificar impuestos requiere capital político y apoyo de grupos; en una autocracia es inmediato pero provoca descontento si es abusivo.
- Cada cambio tiene **retraso** (semanas a meses) y los grupos de interés reaccionan (09): industriales piden rebajas; trabajadores, subsidios a lo básico.

---

## 2. Cómo los impuestos mueven a las empresas privadas

La entrada y expansión de privados depende del **retorno esperado** de cada rubro (18):

```
retorno_esperado = margen_neto × (1 − tasa_empresa[rubro]) − regalías[recurso] − costos_de_riesgo
entra / expande si retorno_esperado ≥ umbral (≈ 8 %) × factor_de_riesgo
```

- **Bajar un impuesto** (empresa, regalía o selectivo) sube el retorno → **entran empresas nuevas** y las existentes **expanden**.
- La respuesta **no es instantánea**: construir instalaciones lleva meses; la inversión es proporcional a `(retorno − umbral)` y está limitada por el **capital disponible**.
- **Costo:** menos recaudación **por unidad** (a corto plazo). Si la base crece lo suficiente, la recaudación total puede compensarlo (curva simplificada: la base crece con `(1 − tasa)^ε`).
- **Sobreoferta:** un incentivo excesivo puede crear exceso de oferta, caída de precios y **quiebras** posteriores (ciclos, 03).

### Ejemplo (orientativo): incentivar metales
Situación: el Estado necesita más acero y aleaciones para astilleros, satélites, estaciones y armas.
1. Baja las **regalías del hierro** (10 % → 2 %) y el **impuesto de empresas en minería** (20 % → 12 %).
2. El retorno esperado de la minería de hierro pasa de ~6 % a ~11 % (> umbral): **entran nuevas empresas** y las existentes amplían.
3. En 6–12 meses la **producción sube** y el **precio del hierro baja** (stock objetivo cubierto), lo que **abarata el acero** y las naves.
4. El Estado **compra más y más barato** (contratos o importación estatal, 03).
5. Costo: menos recaudación inmediata por tonelada; riesgo de **sobreoferta** si el Estado no compra lo suficiente; presión de otros grupos que piden el mismo trato.

---

## 3. Otros instrumentos de incentivo

| Instrumento | Efecto | Costo / riesgo |
|---|---|---|
| **Subsidios por producción** | Suben el margen de un bien o rubro | Gasto fiscal; distorsión |
| **Crédito subsidiado** | Reduce el costo de capital; acelera inversión | Riesgo de impagos; inflación si se financia con emisión |
| **Precio de sostén / compra garantizada** | El Estado asegura una demanda mínima (p. ej. de metales para astilleros) | Compromiso de gasto; acumula stock |
| **Contratos de obra** | Demanda estatal para naves, satélites, estaciones, armas: genera pedidos en cadena | Concentra poder en proveedores |
| **Vacaciones fiscales** | Exención temporal a empresas nuevas en un rubro | Pérdida de ingresos; abuso |
| **Zonas de fomento** | Licencias baratas e infraestructura en un cuerpo/sistema | Desbalance regional (05) |
| **Inversión estatal en producción propia** | Empresas estatales aumentan oferta y empujan precios abajo | Costo administrativo (02) |
| **Inversión estatal en logística** | Puertos, depósitos, flotas estatales de carga, seguridad de rutas: reduce costos de transporte y riesgo (04, 11) | Gasto de capital y mantenimiento |

Los incentivos también pueden aplicarse a la **logística**: reducir el costo y el riesgo del transporte hace rentables rutas y rubros que antes no lo eran.

---

## 4. Mercados grises y bienes éticamente dudosos

### Principio
Cada facción define la **legalidad** de cada bien (L/R/P, 18), según su régimen, cultura y líder (08, 09, 10). Una misma sustancia puede ser legal en un lugar y prohibida en otro. Los sistemas de esta sección ofrecen **ventajas tentadoras con costos reales**, y son configurables como "contenido sensible" en los parámetros de mundo (14).

### 4.1 Sustancias (ficticias)

| Sustancia | Tipo | Beneficio | Costo | Adicción | Legalidad típica |
|---|---|---|---|---|---|
| **Tabaco** | Estimulante suave | Ocio, ingresos fiscales | Salud a largo plazo | Baja-media | Legal/regulado |
| **Vigorán** | Estimulante | Productividad ↑, atención ↑ | Salud ↓, desgaste, dependencia | Media | Regulada |
| **Sosiego** | Sedante | Estrés/malestar ↓ | Productividad ↓ | Media | Regulada |
| **Bruma** | Recreativa | Ocio ↑ | Riesgo social, accidentes | Media | Prohibida (a menudo) |
| **Lumbre** | Narcótico | Ingresos altos | Adicción severa, mortalidad, crimen | **Alta** | Prohibida |

Cada una se produce con recetas propias (18) en **laboratorios químicos** o plantaciones, por empresas **privadas, sindicatos criminales o el propio Estado**.

### 4.2 Uso interno (estilo RimWorld)

El Estado puede adoptar **políticas de uso** sobre sus propios pops, por sector (trabajadores, militares, científicos, administradores):

| Política | Ventaja | Desventaja |
|---|---|---|
| **Programa de estimulantes** (Vigorán) en industria/ejército | Productividad y rendimiento ↑ a corto plazo | Adicción, salud ↓, ausentismo, costo creciente, malestar si se corta |
| **Programa de sedación** (Sosiego) | Menos malestar, menos protestas | Productividad ↓, dependencia, pasividad política |
| **Legalizar y gravar** | Ingresos por impuestos selectivos, menos crimen asociado | Consumo ↑, salud pública, rechazo de grupos conservadores/religiosos |
| **Prohibición estricta** | Menos consumo | Mercado negro, corrupción, sindicatos criminales |

La **dependencia** de un pop es una variable (0–1) que sube con el uso y baja lentamente sin él; la abstinencia provoca malestar y menor productividad. Un brote de adicción puede volverse una **crisis** (05).

### 4.3 Contrabando y comercio gris

El jugador puede **producir y exportar** bienes prohibidos o regulados hacia **facciones con las que hay pacto comercial pero no alianza**, y hacia **zonas francas**.

```
beneficio = volumen × (precio_destino × prima − costo) × (1 − comisión)
p_detección = f(seguridad_destino, volumen, corrupción_aduana_destino, sigilo_flota, inteligencia)
```

- **Precio con prima:** los bienes prohibidos se venden a 3–8× su precio normal.
- **Ingresos negros:** no tributan, pero necesitan lavarse o gastarse en canales grises.
- **Incentivar el contrabando:** el Estado puede **aumentar la producción y la logística** (laboratorios, flotas encubiertas, agentes, sobornos a aduanas) o **tolerar** a privados y criminales ("hacerse el distraído") para participar del negocio.
- **Si te descubren:** incidente diplomático (10), casus belli, sanciones, daño de reputación, y el líder del destino puede tomar represalias (10).
- **Efectos en el destino:** adicción, crimen, corrupción y malestar (05); esto puede **debilitarlo**, pero también **endurecer su postura** hacia vos.
- **Efectos en tu país:** crecen los **sindicatos criminales** (09), la corrupción y la dependencia de ingresos negros; los grupos de interés religiosos o conservadores se oponen.

### 4.4 Zonas francas (estaciones y puntos libres)

Estaciones o puntos de salto **fuera de jurisdicción** (o de facciones permisivas) donde todo es legal: **mercado negro**, centro de contrabando y refugio de piratas y sindicatos.

- El jugador puede **fundar una zona franca**: atrae comercio gris y recauda tasas, pero también criminales y conflictos.
- O puede **atacar o bloquear** las ajenas (11).
- Las zonas francas tienen **bajo nivel de seguridad** (11) y alto riesgo para el transporte.

### 4.5 Otras prácticas dudosas

- **Investigación sin restricciones éticas:** más rápida, con riesgo de escándalo, accidentes y daño de reputación (13).
- **Servidumbre y trabajo forzado:** sistema ya definido en 09.
- **Armas prohibidas y atrocidades:** ver 11.

---

## 5. Leyes de trabajo (decidido: parámetros con trade-offs, tratamiento abstracto)

Parámetros de ley, con efectos agregados en la población. **Sobrio, estilo Victoria 3, sin escenas explícitas.**

| Ley | Opciones | Efectos |
|---|---|---|
| **Edad mínima de trabajo** | Alta (18) · Media (16) · Baja (12–14) · Sin límite | Baja: fuerza laboral ↑ y producción ↑ a corto plazo, pero educación ↓, productividad futura ↓, malestar y daño de **reputación** ↑, rechazo de ciertas culturas. Alta: lo contrario |
| **Jornada laboral** | Corta · Estándar · Larga | Larga: producción ↑, salud y ánimo ↓, accidentes ↑ |
| **Seguridad laboral** | Nula · Básica · Estricta | Estricta: accidentes ↓, costo ↑; nula: ahorro con mortalidad y malestar |
| **Salario mínimo** | Sin · Bajo · Alto | Alto: consumo ↑, desempleo ↑ en rubros marginales |
| **Derechos sindicales** | Prohibidos · Limitados · Libres | Libres: estabilidad social y salarios ↑, poder de sindicatos |
| **Servidumbre / trabajo forzado** | Prohibido · Permitido (09) | Producción ↑, revueltas, sanciones, reputación ↓ |
| **Servicio obligatorio** | Ninguno · Militar · Civil | Fuerza armada/obras ↑, mano de obra ↓ |

- La **cultura** y la **especie** influyen en la aceptación (08).
- Las leyes más duras generan **malestar** y pueden llevar a huelgas, rebeliones o presión externa (05, 09, 10).
- Son **opcionales como contenido sensible** (se pueden limitar o deshabilitar en la configuración del mundo).

---

## 6. Interfaz

- **Árbol de impuestos** con heredado/override y un panel "**qué pasaría si**" (proyección de recaudación, retorno esperado por rubro, reacción de grupos de interés).
- Panel de **mercados grises**: legalidad por bien y facción, rutas de contrabando, riesgo estimado y reputación.
- Indicadores de **dependencia** y salud por sector de pops.
- Alertas de **sobreoferta**, **déficit**, **aduanas** y **descubrimiento** (16).

## Conexiones

03 (reglas económicas) · 05 (malestar, salud, adicción) · 08 (cultura y especie) · 09 (grupos de interés, servidumbre, sindicatos criminales) · 10 (incidentes, sanciones) · 11 (seguridad, aduanas, zonas francas) · 13 (investigación sin ética) · 14 (parámetros de contenido sensible) · 16 (UI) · 18 (bienes y recetas).

## Preguntas abiertas

- ¿Cuántos niveles de capital político cuesta cada cambio de ley o impuesto por régimen?
- Valores concretos de prima, p_detección y comisión del contrabando.
- ¿Las zonas francas son entidades propias (facciones) o instalaciones de un Estado?
- ¿Cómo se modela la dependencia a nivel de pop (variable continua por sustancia, o un solo índice)?
- ¿Se puede "legalizar y gravar" con reducción real del mercado negro, o sólo lo desplaza?
