# 09 — Facciones y política interna

> v0.3 · Alineado con 19 (impuestos, incentivos, mercados grises, leyes laborales, servidumbre), 20 (7 regímenes, líderes, sucesión, espionaje) y 22 (grupos de interés de la Tierra en el MVP).

## Tipos de facción

| Tipo | Ejemplos | Tiene territorio |
|---|---|---|
| Estados | Repúblicas, imperios, federaciones, regímenes colectivistas, teocracias | Sí |
| Corporaciones | Mineras, navieras, aeroespaciales, conglomerados (crecen desde empresas privadas) | A veces (concesiones, estaciones) |
| Órdenes religiosas | Monasterios, cruzadas, misiones | A veces |
| Compañías mercenarias | — | No (bases) |
| Piratas / saqueadores / señores de la guerra | — (desde la Fase 2) | **Bases reales**, zonas sin ley |
| Sindicatos criminales | Contrabando, extorsión | Infiltrados |
| Movimientos rebeldes / separatistas | — | Regiones/colonias en disputa |
| Nómadas | Flotas Drel | No |
| Colectivos de IA | Sintéticos independientes | Variable |
| Estados clientes / protectorados | Dependientes de una potencia | Sí |

**Ciclo de vida:** las facciones **nacen** (secesión, fundación, cisma, corporación que se independiza), **crecen**, se **transforman** (revoluciones) y **mueren** (conquista, colapso, absorción). El mapa político cambia constantemente.

> El jugador siempre es un **Estado** (00). Al empezar (inicio por defecto), **su planeta capital está unificado bajo su gobierno** y es un único nodo, sin regiones ni bloques (02, 05). Otras facciones aparecen en otros cuerpos del sistema, en otros sistemas (Fase 2) o como rebeldes internos. **En la Fase 1 no hay facciones armadas en el sistema local.**

---

## Régimen del jugador y de las facciones

Hay **7 regímenes** (definición completa, pesos de líder `w` e inercias en 20). Se combinan con ejes que los modulan:

| Eje | Rango |
|---|---|
| Distribución del poder | Autocracia ↔ Oligarquía ↔ Democracia |
| Fuente de legitimidad | Tradición/herencia · Fe · Voluntad popular · Fuerza militar · Mérito/tecnocracia · Contrato corporativo |
| Centralización | Unitario ↔ Federal ↔ Confederación |
| **Modelo económico** | **Planificado (todo estatal) ↔ Mixto ↔ Libre mercado** — determina qué empresas existen y qué herramientas fiscales están disponibles (03, 19) |
| Ciudadanía | Quiénes tienen derechos (especie, clase, todos) |
| **Trabajo** | Libre ↔ con servidumbre/trabajo forzado (opcional, ver abajo y 19) |

Las combinaciones generan nombres ("Teocracia", "Junta militar", "República corporativa", "Estado planificado", "Colmena") y determinan herramientas disponibles. **El jugador elige su régimen al empezar y puede reformarlo** con capital político y apoyo de grupos de interés; cada régimen tiene **inercia**: los cambios bruscos generan resistencia (20).

---

## Líder y régimen (modelo EU4 + CK3)

Cada facción tiene un **líder** (notable, 06) cuyas cualidades, junto con las de la especie y el régimen, definen su comportamiento (detalle en 20; diplomacia en 10).

- **Peso del líder (`w`) según el régimen:** alto en autocracias, monarquías y teocracias; medio en oligarquías; **bajo en democracias**, donde los cambios radicales requieren apoyo de grupos de interés y tiempo. Valores exactos por régimen en 20.
- **El líder del propio jugador** también cuenta: aporta modificadores al Estado (administración, diplomacia, economía, militar, estabilidad), puede tener rasgos buenos o problemáticos y es sujeto a sucesión.
- **Candidatos y facciones de poder:** cada régimen genera un **pool de 3–5 candidatos** (heredero, jefe militar, líder opositor, candidato electoral…) respaldados por grupos de interés. Quién llega al poder depende de las reglas del régimen y de la correlación de fuerzas (20).
- **Cambio de líder:** natural (muerte, vejez), sucesión, elecciones, golpe, revolución, asesinato. Los cambios provocados externamente (espionaje) tienen riesgos de descubrimiento y consecuencias (10, 20).

---

## Leyes y políticas

Categorías: economía, **trabajo**, ciudadanía, migración, religión, seguridad, justicia, ejército, derechos de sintéticos, uso de recursos, medio ambiente, **propiedad (rubros reservados al Estado)**, **impuestos e incentivos** (19).

- Cambiar leyes requiere **capital político** y apoyo de grupos de interés; lleva tiempo.
- **Leyes económicas y laborales** (19): impuestos por categoría de bien y actividad, incentivos fiscales, jornada, salario mínimo, sindicatos, servidumbre. Las regulaciones excesivas generan **mercados grises** (03, 19).
- Leyes de emergencia (estilo Frostpunk) en crisis: racionamiento, toque de queda, conscripción — fáciles de imponer, difíciles de quitar.

---

## Grupos de interés (inspirado en Victoria 3)

Ejemplos: Militares · **Industriales/Corporaciones** · **Comerciantes y navieras** · Terratenientes/agrarios · Clero · Intelectuales · Trabajadores y sindicatos · Burocracia · grupos por especie o cultura.

**En la Fase 1 (MVP) hay sólo 4 grupos de interés más la población general** (22; p. ej. industria, ciencia, trabajadores/sindicatos y seguridad/nacionalismo), para mantener el modelo legible; el catálogo completo se activa al crecer la civilización.

Cada uno tiene:
- **Influencia** (riqueza, miembros, cargos).
- **Ideología** (qué leyes quiere).
- **Aprobación** del gobierno.
- **Candidato preferido** al liderazgo.

Un grupo con mucha influencia y baja aprobación es un riesgo: bloquea reformas, financia opositores, organiza golpes. Las **corporaciones grandes** ganan poder político a medida que la economía privada crece (03). En la Fase 1 su aprobación se combina con el **apoyo público** en la fracción de presupuesto destinada al espacio (05, 22).

---

## Legitimidad, estabilidad y corrupción

- **Legitimidad:** cuánto se acepta al gobierno según la fuente de legitimidad del régimen.
- **Estabilidad:** agregado de legitimidad, malestar de pops y aprobación de grupos.
- **Corrupción:** crece con la sobreextensión (02) y la falta de supervisión; reduce eficiencia, desvía recursos, habilita contrabando y mercados grises (19). Combatirla cuesta recursos y enemigos.

---

## Servidumbre y trabajo forzado (sistema opcional)

El jugador puede **habilitar o no** el sistema en la partida (dependiendo del régimen y la cultura). Las leyes laborales y sus parámetros están en 19.

- **Qué es:** pops sometidos (por conquista, deuda, casta o ley) con derechos limitados, que trabajan bajo coacción.
- **Efectos:** mayor producción y menor costo salarial a corto plazo.
- **Consecuencias:** malestar y **revueltas** de los sometidos, **sanciones y daño de reputación** con facciones de afinidad opuesta, riesgo de fuga, presión de grupos de interés, estigma histórico y legados en la crónica (15).
- **Abolición:** reformas costosas, con resistencia de beneficiarios; tras una abolición, tensiones persistentes.
- Es una herramienta tentadora para el estilo imperialista (03): conquistar pueblos y explotarlos tiene un precio político a largo plazo.

---

## Crisis internas

- **Crisis de sucesión:** heredero dudoso, varios pretendientes (06, 20).
- **Golpe de Estado:** grupo con mucha influencia y baja aprobación (típicamente militares).
- **Revolución:** levantamiento popular por malestar extremo (05).
- **Secesión:** una región o colonia declara independencia (requiere regiones o colonias grandes; no ocurre en la Tierra del MVP, 05).
- **Guerra civil.**
- **Conflicto con megacorporaciones:** una corporación tan poderosa que se enfrenta al Estado o se independiza.
- **Huelgas y boicot** (nivel 3 de la escalera de malestar): la crisis interna típica de la Fase 1 (05).

---

## Guerra civil (detalle)

### Disparadores
Escalada de malestar · disputa de sucesión · golpe a medias · división ideológica · secesión de una porción grande del territorio.

### Formación de bandos
Cada región, flota, unidad y notable **elige bando** según lealtad personal, ideología, grupo de interés, presión local y cálculo de quién va a ganar.

→ El jugador puede **perder media flota** si un almirante deserta con sus naves.

### Intervención externa
Otras facciones apoyan bandos (armas, dinero, voluntarios) → guerras por delegación. También aprovechan para atacar. El jugador puede **financiar o apoyar** un bando ajeno para cambiar el liderazgo (10).

### Resoluciones posibles
Victoria de un bando (purgas, reformas, venganzas) · acuerdo negociado (federalización, reparto de poder) · **partición** (nueva facción permanente) · conflicto congelado.

### Para el jugador
- Las facciones IA también sufren guerras civiles → oportunidades (apoyar un bando, recoger los pedazos, recibir refugiados, contratar desertores).
- Si el jugador pierde su guerra civil, puede **seguir con el bando perdedor** (remanente en el exilio).

> Fuera del alcance del MVP (21): la Tierra sin regiones no tiene guerra civil.

---

## Espionaje y subversión

- **Agentes** (notables con intriga) y **redes** en territorio ajeno.
- Acciones: reunir información (incluida la de líderes, niveles N0–N3, 20), robar planos y datos de relevamiento, **robo de tecnología** (fuente de investigación, 13), sabotaje (incluyendo puntos de salto e instalaciones), incitar malestar, **apoyar golpes, asesinar líderes**, sobornos, desinformación, financiar rebeldes, provocar incidentes.
- Cada operación se resuelve con `p_éxito` y `p_descubrimiento` independientes (10, 20).
- **Contrainteligencia:** detectar redes, agentes dobles.
- Ser descubierto: incidente diplomático, casus belli.

## Personalidad y memoria de facciones IA

Personalidad derivada de **especie + cultura + régimen + líder** (4 ejes Ag/Ap/Fi/Co, 08, 20). Memoria de agravios y favores (10). Objetivos de largo plazo que cambian con los líderes.

## Conexiones

02 (capacidad administrativa, inicios) · 03 (política económica, empresas) · 05 (malestar, servidumbre, grupos de interés de la Tierra) · 06 (líderes, notables, sucesión) · 08 (especies, regímenes naturales) · 10 (diplomacia, golpes y asesinatos) · 11 (guerra, piratas) · 13 (robo de tecnología) · 15 (historia de facciones) · 19 (impuestos, laborales, mercados grises) · 20 (regímenes, líderes) · 22 (grupos de interés iniciales).

## Preguntas abiertas

- ¿Elecciones simuladas con partidos? (Fuera del MVP.)
- ¿Qué 4 grupos exactos y qué pesos iniciales usa la Tierra del MVP? (22 propone; calibrar.)
- ¿La servidumbre es un parámetro de partida (on/off) o una ley dentro del juego (siempre disponible, con costos)? (19 propone ley con costos.)
- Lista de candidatos al liderazgo: ¿se generan por régimen o por facción? (20: pool de 3–5 por régimen.)
