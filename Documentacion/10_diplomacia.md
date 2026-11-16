# 10 — Diplomacia

> v0.3 · Alineado con 20 (líderes, regímenes, niveles de conocimiento N0–N3, fórmulas de espionaje) y 02 (primer contacto transparente). En el MVP no hay otras civilizaciones (la Fase 1 termina al cruzar el punto de salto): este documento describe el sistema completo para la Fase 2 en adelante.

## Principio

La diplomacia es un **sistema entre todos los actores**, no un menú entre el jugador y la IA. Las facciones forman alianzas, se traicionan y guerrean entre sí aunque el jugador no exista.

El modelo mezcla dos referentes: **Europa Universalis IV** (relaciones, tratados, coaliciones, casus belli) y **Crusader Kings 3** (el **líder** de cada facción importa: sus rasgos, ambiciones y fallas moldean la política exterior). A diferencia de Aurora, **el estado de las relaciones es legible**: siempre se ve cuánto nos aprecian, hacia dónde va y por qué.

---

## Contacto, relevancia y reputación

- **Contacto:** sin detección no hay diplomacia. Primero hay que **encontrar** a la otra facción (relevamiento, sensores, comerciantes, rumores).
- **Relevancia:** cuánto le importa una facción a otra (poder, cercanía, interdependencia económica). Por debajo de un umbral no hay canal diplomático formal.
- **Reputación:** por facción y por grupo/ideología, más etiquetas globales ("traicionero", "cumple tratados", "comerciante confiable", "carnicero"). Sólo la conocen quienes **recibieron la información** (01).
- **Banda de equilibrio:** el poder de las facciones contactables está limitado respecto del tuyo (02); por eso las **coaliciones** son una respuesta viable al rival más fuerte.

---

## Relación visible

Para cada facción contactada, la interfaz muestra un **indicador de relación** con su **tendencia** (sube / estable / baja) y un **desglose** ("¿por qué nos odian/aprecian?"): especie y cultura, régimen, líder, agravios, intereses, regalos recientes, objetivo común (16, 20). Evita el problema de Aurora, donde no está claro cuándo mejoran o empeoran las relaciones.

---

## Líderes y personalidad de las facciones

Cada facción (Estado, civilización, planeta independiente) tiene un **líder**: un notable (06) con **rasgos, ambiciones, habilidades y valores**. Su actitud hacia otras facciones surge de tres capas:

| Capa | Qué aporta | Cambia… |
|---|---|---|
| **Especie / cultura** | Disposición base en los 4 ejes Ag/Ap/Fi/Co (08) | Muy lento |
| **Régimen** | Cuánto margen tiene el líder y cuán rápido puede cambiar el rumbo (09, 20) | Con reformas |
| **Líder** | Rasgos personales (15 pares de rasgos, 20): carismático, paranoico, ambicioso, pacífico, cruel, codicioso, religioso… | Con cada cambio de líder |

Ejemplo: una especie amistosa con un líder cruel y xenófobo puede ser **hostil con vos**; en cambio, una especie recelosa con un líder conciliador puede ser un buen socio.

### Peso del líder según el régimen

```
postura = disposición_base × (1 − w) + personalidad_líder × w
```

`w` es el **peso del líder** del régimen (valores por régimen en 20). Resumen:

| Régimen | Peso del líder | Rigidez diplomática |
|---|---|---|
| Autocracia / monarquía / teocracia | Alto | Baja: puede pasar de aliado a enemigo con rapidez |
| Junta militar | Medio-alto | Baja-media |
| Oligarquía / república corporativa | Medio | Media: depende de los grupos de interés |
| Democracia | Bajo | **Alta**: los cambios radicales requieren apoyo de grupos de interés y tiempo |
| Colectivista / colmena (p. ej. Ossari) | Según la estructura innata (08): a veces casi nulo, a veces concentrado en una figura | Variable |

**Rigidez diplomática (inercia):** en regímenes con muchos contrapesos, pasar de aliado a guerra (o viceversa) es difícil: exige varios eventos, caída de aprobación y tiempo. Las relaciones tienen **inercia** (20).

### Cambio de líder
El líder cambia por:
- **Causas naturales:** vejez, enfermedad, muerte, abdicación.
- **Sucesión** según el régimen (pool de 3–5 candidatos: heredero, designado, consejo; 20).
- **Elecciones** (democracias).
- **Golpe de Estado** o revolución.
- **Asesinato.**

El jugador puede **influir** (ver "Acciones de influencia"). Los efectos del nuevo líder dependen de quién queda como candidato: **hay probabilidades de que el próximo sea más razonable, pero no garantía**. A veces el reemplazo es peor.

### Acciones de influencia

| Acción | Efecto | Riesgo |
|---|---|---|
| **Diplomacia personal** | Regalos, honores, matrimonios, favores: mejora opinión del líder hacia vos | Costo; el líder puede ser incorruptible |
| **Soborno / chantaje** | Cambia decisiones puntuales | Descubrimiento, escándalo |
| **Apoyar a un candidato o a la oposición** | Inclina elecciones, sucesión o un golpe | Si falla, incidente diplomático; si gana, el nuevo líder **te debe un favor… o te lo resiente** |
| **Financiar un golpe de Estado** | Derrocar al líder; la probabilidad depende de la inestabilidad, grupos de interés y régimen | Guerra civil, descubrimiento, casus belli |
| **Asesinato del líder** | Fuerza una sucesión | Alto riesgo de ser descubierto (casus belli, reputación); puede **crear un mártir**, unir al pueblo y endurecer la postura |
| **Propaganda / desinformación** | Cambia la opinión de pops o grupos de interés sobre el líder o sobre vos | Contrapropaganda |
| **Robo de tecnología** | Operación de espionaje que obtiene avances de investigación de otra facción (13) | Descubrimiento, incidente, casus belli |

### Cómo se resuelven (decidido: operación de espionaje con probabilidad)

Las acciones de golpe, asesinato, apoyo a candidatos, soborno y robo de tecnología son **operaciones de espionaje**. El jugador asigna **agentes** (notables con intriga, 06), **presupuesto** y un **objetivo**, y la operación se resuelve con dos probabilidades **independientes** (fórmulas y parámetros base en 20):

```
p_éxito         = f(habilidad_agentes, presupuesto, red_de_inteligencia,
                    inestabilidad_objetivo, peso_líder/régimen, contrainteligencia)
p_descubrimiento = f(contrainteligencia, tamaño_operación, régimen, azar)
```

| Resultado | Consecuencias |
|---|---|
| Éxito sin ser descubierto | El cambio ocurre; no hay atribución (aunque pueden haber sospechas) |
| Éxito y descubierto | El cambio ocurre pero hay **incidente diplomático / casus belli**; el nuevo líder puede resentirlo |
| Fracaso sin descubrir | Se pierden recursos; el objetivo permanece |
| Fracaso y descubierto | Agentes capturados, casus belli, reputación dañada |

- Las operaciones **tardan** (semanas a meses) y se muestran como un informe con el resultado y el "por qué".
- Las probabilidades se **estiman** antes de lanzar la operación (con rangos, según tu inteligencia).
- **Régimen:** en democracias es difícil provocar cambios radicales pero hay más candidatos y elecciones que inclinar; en autocracias es más fácil derribar a un líder pero la sucesión puede ser sangrienta.

### Información sobre el líder (niveles de conocimiento N0–N3)
Los rasgos y ambiciones de un líder ajeno se conocen según tu **inteligencia, embajadas y contacto** (20):

| Nivel | Qué ves |
|---|---|
| **N0** | Sólo el cargo |
| **N1** | Nombre y un rasgo visible |
| **N2** | Rasgos y ejes de personalidad aproximados (rangos) |
| **N3** | Ficha completa |

La ficha de líder y el botón **"¿por qué nos odian/aprecian?"** muestran el desglose: especie/cultura, régimen, líder, agravios, intereses (16).

### Líderes y guerra
Un líder ambicioso o paranoico busca conflicto con más facilidad; uno pacífico, tratados. En regímenes de peso alto, la **muerte de un líder en guerra** puede provocar un alto el fuego, una crisis de sucesión o más radicalización.

---

## Modelo de relación (multidimensional)

| Dimensión | Fuente |
|---|---|
| **Confianza** | Historial de cumplimiento |
| **Temor** | Poder militar relativo y cercanía |
| **Afinidad** | Ideología, cultura, especie, religión, régimen, **compatibilidad entre líderes** |
| **Interés** | Comercio, enemigos comunes, recursos complementarios |
| **Agravios** | Memoria de ofensas, con decaimiento según cultura (los Kethari recuerdan siglos) |
| **Deudas / favores** | Ayudas pasadas, deuda financiera, regalos |

La **opinión** (el indicador visible) resume estas dimensiones, pero las decisiones se toman por **utilidad esperada** de cada acuerdo, ajustada por la personalidad del líder y el régimen. Una facción puede odiarte y aun así comerciar con vos si le conviene.

---

## Canales y actores

- **Enviados y embajadores:** notables con habilidad diplomática y rasgos propios. Las **embajadas** dan información (incluida la del líder, subiendo su nivel N) y reducen fricción.
- **Distancia:** en escala interestelar, los enviados negocian con un **mandato** (márgenes de lo que pueden aceptar). Pueden excederse, fracasar o ser sobornados. La latencia de información (01) afecta los tiempos.

---

## Acuerdos

- **Comerciales:** pactos de comercio (permiten que **las empresas de ambos lados comercien**), aranceles, cuotas, puertos autorizados, acceso a mercados.
- **Estado–Estado:** contratos de suministro, trueques de bienes y tecnología (especialmente con regímenes totalmente estatales).
- **Seguridad:** no agresión, pacto defensivo, alianza, acceso militar, **patrullas conjuntas**, derechos de tránsito por puntos de salto, acuerdos sobre minas y peajes.
- **Territoriales:** fronteras, reclamos, uso compartido de recursos (un cinturón de asteroides), condominio.
- **Vasallaje, tributo, protectorado.**
- **Tecnológicos:** intercambio, **licencias de planos y de tecnología** (fuente de investigación, 13).
- **Migración, extradición, refugiados.**
- **Alto el fuego, tratados de paz.**
- **Federación / unión.**

Cada acuerdo tiene **duración, condiciones y verificación**. Romperlos daña la reputación con todos los que se enteren. Un cambio de líder puede **renovar, cuestionar o repudiar** acuerdos previos (más fácil en regímenes de peso alto).

---

## Acuerdos de servicios

Para los estilos *cliente* e *imperialista* (03), una facción puede **contratar o proveer**:

| Servicio | Qué se obtiene | Contraprestación típica |
|---|---|---|
| **Protección militar** | Flotas, patrullas o garantía de defensa en tu territorio | Tributo, bases, alineamiento, concesiones |
| **Recursos** | Suministro de minerales, combustible, bienes | Pago, deuda, derechos de extracción |
| **Desarrollo** | Inversión e infraestructura (puertos, instalaciones, colonias) | Participación en ganancias, concesiones |
| **Tecnología** | Licencia, transferencia, formación de científicos | Pago, exclusividad, dependencia |

Riesgos: **dependencia**, pérdida de autonomía, cláusulas de salida costosas, que el proveedor cambie de política (o de **líder**) y se desentienda. Se puede progresar de cliente a socio y, con éxito, a independiente.

---

## Negociación

- Ofertas compuestas por ítems: dinero, territorio, recursos, tecnología, concesiones, rehenes, matrimonios (gobiernos dinásticos), garantías.
- La IA evalúa con utilidad + personalidad del líder + relación + **prestigio** (no aceptar algo humillante aunque convenga).
- Contraofertas, ultimátums, amenazas creíbles (o no).

## Casus belli y objetivos de guerra

- Surgen de agravios: incidentes fronterizos, **violación de puntos de salto o espacio**, tratados rotos, atrocidades, **asesinato o golpe descubierto**, **robo de tecnología descubierto**, persecución de correligionarios, reclamos históricos (14).
- Guerra sin casus belli → pérdida de legitimidad y reputación.
- Objetivos: territorio, recursos, **tecnología y conocimiento**, tributo, cambio de régimen o de líder, liberación, (extremo) exterminio.

## Bloques y organizaciones

Alianzas, bloques comerciales, consejos con votación, federaciones, instituciones interestelares tardías. Creados por la IA o por el jugador.

**Equilibrio de poder:** si alguien crece agresivamente, los demás tienden a coaligarse en su contra (estilo EU4). Con la banda de equilibrio (02), una coalición del jugador con otras facciones es viable contra el rival más fuerte (hasta ×1,5 del jugador).

---

## Diplomacia no estatal

- Pagar "protección" a piratas.
- Contratar mercenarios.
- Negociar concesiones con corporaciones (propias o extranjeras).
- Apoyar rebeldes en territorio ajeno.
- Acuerdos con órdenes religiosas.

## Primer contacto (transparente)

Fases: **detección → observación → contacto → traducción → relaciones**. Es un proceso **legible**, no una lotería (02):

1. **Protocolo de paz:** el jugador puede **desactivar armas** (postura "no hostil") y **activar su transmisor** (señal de paz) (tecnología ADM-2.3, 13). Las facciones neutrales que lo reciben **no atacan** salvo líderes muy agresivos o xenófobos.
2. **Riesgo inicial de hostilidad:**
   ```
   p_ataque_inicial = clamp( 0,15 + 0,12×Ag_postura − 0,10×Ap_postura − 0,25×señal_de_paz, 0,02, 0,70 )
   ```
   `Ag_postura` y `Ap_postura` son los ejes de la postura del líder hacia vos (−3 a +3, 20).
3. **Regalos y gestos:** enviar recursos, datos o tecnología modesta mejora la actitud (y se refleja en la tendencia de la relación).
4. **Objetivo común:** con comunicación establecida, ambas partes pueden **fijar un objetivo** (comercio, alianza defensiva, exploración conjunta, no agresión).
5. **Idiomas por fases:** sin lenguaje (gestos y señales) → traducción parcial (mensajes simples, comercio básico) → lenguaje común (diplomacia completa). El tiempo depende del **factor de idioma** de la especie y de la xenología (COM-1.2): ≈ 1–3 años (08, 13).

- La psicología de cada especie y del **líder** influye en cómo interpreta las acciones del otro.
- Riesgo de malinterpretación: una maniobra de escolta puede leerse como agresión.
- **Contacto por puntos de salto:** una civilización que abre un punto de salto cerca de otra genera anomalías y tensiones antes de la comunicación (11, 14).

## Incidentes

Emergentes: una patrulla tuya ataca a un mercader neutral; un espía es capturado; refugiados cruzan la frontera; minas en un punto de salto hunden una nave extranjera; una incursión sin declaración de guerra; un asesinato atribuido. Cómo se gestionan define relaciones.

## Conexiones

01 (información y latencia) · 02 (primer contacto, banda de equilibrio) · 03 (pactos, embargos, acuerdos de servicios) · 06 (líderes, enviados, agentes) · 08 (afinidad, idiomas, disposición base) · 09 (regímenes, golpes, espionaje) · 11 (guerra y paz, seguridad) · 13 (licencias, robo de tecnología, ADM-2.3, COM-1.2) · 16 (relación visible) · 20 (fórmulas y niveles de conocimiento).

## Preguntas abiertas

- ¿Qué tan visible es el razonamiento de la IA para el jugador más allá del desglose de la relación?
- ¿Instituciones galácticas (tipo senado) sí o no?
- Cláusulas de salida y penalizaciones en acuerdos de servicios.
- Calibrar `p_éxito`, `p_descubrimiento` y `p_ataque_inicial` con prototipos (20).
