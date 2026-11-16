# 20 — Líderes, regímenes y rasgos

> v0.1 · Concreta el modelo EU4 + CK3 descrito en 06 (personajes), 09 (política interna) y 10 (diplomacia). Todos los números son **valores iniciales a calibrar** con prototipos.

## Decisiones de diseño (resumen)

| Tema | Resolución |
|---|---|
| Rasgos | **Lista fija con opuestos** (~30 rasgos en 15 pares), 2–4 por líder, con efectos numéricos claros. |
| Regímenes | **7 arquetipos** con peso del líder, inercia y regla de sucesión propios. |
| Sucesión | **Según el régimen**: cada régimen genera candidatos y elige con su propia regla. |
| Visibilidad | La ficha de un líder ajeno **se descubre con inteligencia** (4 niveles de conocimiento). |

---

## 1. Ficha de un líder

| Bloque | Contenido |
|---|---|
| **Identidad** | Nombre, especie, edad, salud, cultura, familia (herederos). |
| **Estadísticas de gobierno** (0–6) | Administración · Diplomacia · Economía · Militar. Salen de las habilidades del notable (06) y los rasgos. |
| **Rasgos** | 2–4 innatos + hasta 2 adquiridos (ver §2). |
| **Ambición** | Una de: poder, riqueza, expansión, fe, seguridad, conocimiento, legado. Orienta decisiones a largo plazo. |
| **Vector de personalidad** | Cuatro ejes derivados de los rasgos (§2). |
| **Apoyo** | Grupos de interés que lo respaldan y su influencia (09). |

Los modificadores de las estadísticas de gobierno se aplican al Estado (ej.: Administración +1 ≈ +5% de capacidad administrativa; Economía +1 ≈ +3% de recaudación eficiente; cifras a calibrar).

---

## 2. Rasgos

### Reglas
- Un líder tiene **2–4 rasgos innatos**; los opuestos **no pueden coexistir**.
- Los **rasgos adquiridos** (hasta 2) surgen de eventos: *Veterano, Traumatizado, Corrupto, Celebridad, Fanático, Vengativo contra X, Reformador*. Pueden pisar un rasgo innato débil.
- **Herencia** (regímenes hereditarios, propuesta): cada rasgo del progenitor se hereda con 25%; el resto se sortea según el perfil cultural de la especie.
- Los rasgos de una especie no son uniformes: cada cultura sesga las probabilidades de sorteo (08).

### Vector de personalidad (4 ejes, −3 a +3)

| Eje | Significa | Afecta a |
|---|---|---|
| **Ag — Agresividad** | Propensión a la fuerza y al conflicto | Casus belli, guerras, respuesta a incidentes |
| **Ap — Apertura** | Disposición hacia otras facciones y especies | Tratados, comercio, migración, actitud en primer contacto |
| **Fi — Fiabilidad** | Cumplimiento de acuerdos y palabra | Probabilidad de romper tratados, reputación |
| **Co — Codicia** | Apetito de riqueza y recursos | Demandas en negociación, corrupción, soborno |

`P_eje = clamp(Σ efectos de rasgos + modificador de especie, −3, +3)`

Este vector es la `personalidad_líder` de la fórmula de 10:

```
postura_eje = disposición_base_especie_eje × (1 − peso_líder) + P_eje × peso_líder
```

### Los 15 pares de rasgos

| # | Rasgo A | Efectos | Rasgo B (opuesto) | Efectos |
|---|---|---|---|---|
| 1 | **Belicoso** | Ag +2; Militar +1 | **Pacifista** | Ag −2; Diplomacia +1 |
| 2 | **Ambicioso** | Ag +1, Co +1; Administración +1; estabilidad −5% | **Conformista** | Ag −1; Administración −1; estabilidad +5% |
| 3 | **Paranoico** | Ap −2, Fi −1; contrainteligencia +1; p_éxito de espionaje contra él −0,10 | **Confiado** | Ap +1, Fi +1; p_éxito de espionaje contra él +0,10 |
| 4 | **Xenófobo** | Ap −2; malestar de minorías +; aprobación de grupos nacionalistas + | **Cosmopolita** | Ap +2; integración de especies + |
| 5 | **Carismático** | Diplomacia +2; aprobación de pops + | **Retraído** | Diplomacia −1; aprobación − |
| 6 | **Cruel** | Ag +1; malestar +; supresión de revueltas + | **Compasivo** | Ag −1; aprobación +; revueltas más fuertes |
| 7 | **Codicioso** | Co +2; corrupción + | **Austero** | Co −2; corrupción −; gasto de corte − |
| 8 | **Honorable** | Fi +2; reputación de cumplimiento + | **Traicionero** | Fi −2; mejora p_éxito de sus propias operaciones |
| 9 | **Devoto** | Legitimidad en teocracias +; rechazo a tecnologías "prohibidas" | **Escéptico** | Investigación +; legitimidad en teocracias − |
| 10 | **Trabajador** | Administración +1 | **Perezoso** | Administración −1; eventos de negligencia |
| 11 | **Erudito** | Investigación +; Economía +1 en tecnocracias | **Ignorante** | Investigación −; sesgos en decisiones |
| 12 | **Astuto** | Intriga +2; p_éxito propia +0,05 | **Ingenuo** | Intriga −1; más vulnerable a engaños |
| 13 | **Valiente** | Militar +1; no rechaza guerras difíciles | **Cobarde** | Militar −1; tiende a ceder |
| 14 | **Temperamental** | Sobrerreacciona a incidentes (×1,5 en escalada) | **Sereno** | Incidentes se desescalan (×0,7) |
| 15 | **Reformista** | Inercia del régimen −25% para cambios de leyes | **Tradicionalista** | Inercia +25%; estabilidad + |

(Las bonificaciones son iniciales y se ajustan con el balance. Cada rasgo tiene un icono y una frase breve para la UI.)

---

## 3. Regímenes (7 arquetipos)

| Régimen | Peso del líder `w` | Inercia (meses para un cambio radical)¹ | Regla de sucesión | Fuente de legitimidad |
|---|---|---|---|---|
| **Democracia** | 0,25 | 12 | Elecciones cada 4–6 años; 3–5 candidatos | Voluntad popular |
| **República oligárquica** | 0,45 | 8 | Consejo de grupos de interés; mandato largo o vitalicio | Contrato entre élites |
| **Monarquía** | 0,75 | 3 | Hereditaria (primogenitura o designación) | Tradición / herencia |
| **Junta militar** | 0,65 | 2 | El jefe militar con más apoyo; relevo por golpe interno | Fuerza |
| **Teocracia** | 0,70 | 4 | Conclave del clero | Fe |
| **Tecnocracia** | 0,35 | 8 | Designación por mérito (consejo científico, evaluaciones periódicas) | Mérito |
| **Corporativismo** | 0,50 | 6 | Mayor tenedor o directorio; mandato condicionado a resultados | Contrato corporativo |

¹ **Cambio radical** = pasar de alianza a guerra (o a la inversa), repudiar tratados clave, un giro económico. Requiere que se acumule presión durante esos meses y, en regímenes con grupos de interés fuertes, **aprobación ≥ 50%** de los grupos relevantes. Un líder con *Reformista* reduce los meses un 25%; con *Tradicionalista*, los aumenta.

- La combinación de **modelo económico** (planificado / mixto / libre, 09) es independiente: una teocracia puede ser planificada o de mercado.
- **Variantes de especie:** regímenes colectivos o colmena (Ossari, 08) usan `w` propio (casi nulo o muy concentrado) y su propia regla de sucesión, definida en la ficha de especie.
- **Estado del jugador:** mismas reglas; el líder del jugador no se controla pero sus rasgos modifican al Estado y está sujeto a sucesión (06).

---

## 4. Sucesión y candidatos

### Cuándo ocurre
Muerte, vejez, abdicación, elecciones programadas, golpe, revolución, asesinato o crisis de legitimidad.

### Quién se postula
Cuando se abre la vacante el régimen genera un **conjunto de candidatos** (3–5) a partir de los notables de la facción, más herederos si aplica. Cada candidato tiene rasgos, estadísticas, ambición y **grupos que lo respaldan**.

```
peso_candidato = Σ_g ( influencia_g × afinidad(g, candidato) ) × regla_régimen × (1 + 0,5 × apoyo_externo)
```

`apoyo_externo` va de 0 a 1 y refleja el respaldo logrado con operaciones de espionaje (10).

| Régimen | Cómo se elige entre candidatos |
|---|---|
| Democracia | Votos ponderados por grupos y pops; ruido electoral moderado |
| República oligárquica | Votación del consejo; gana el de mayor peso, con alta inercia |
| Monarquía | El heredero designado gana salvo **disputa** (heredero dudoso, menor, impopular): entonces se abre crisis con pretendientes |
| Junta militar | El jefe con más unidades leales; alta probabilidad de violencia |
| Teocracia | Conclave: el de mayor peso entre el clero, con sesgo por devoción |
| Tecnocracia | El de mayor Administración/Economía/Investigación |
| Corporativismo | El de mayor participación o desempeño en la gestión |

La elección sigue una **selección probabilística** (más peso = más probabilidad). La "temperatura" depende del régimen: baja en monarquías (casi determinista), alta en juntas o en crisis.

### ¿Será el próximo más razonable?
El sistema **no garantiza** mejora: el candidato proviene del mismo pool de la facción. Por eso:
- Un líder odiado por pocos puede tener **sucesores del mismo bando** (los que lo respaldaban).
- Un golpe o asesinato tiende a **cambiar el bando** si hay una oposición fuerte.
- El jugador puede **inclinar los pesos** con `apoyo_externo` (10), sabiendo que el candidato apoyado puede **deberle el favor o resentirlo** (según sus rasgos: *Honorable* / *Traicionero*).
- Si un asesinato es atribuido, el sucesor puede adquirir **Vengativo contra X**.

### Crisis de sucesión
Ocurre cuando dos candidatos tienen pesos cercanos (diferencia < 15%) o falta heredero: se activan **pretendientes** y, si la facción es inestable, **guerra civil** (09).

---

## 5. Conocimiento de líderes ajenos

La ficha de un líder extranjero se revela por niveles.

| Nivel | Qué ves | Cómo se alcanza |
|---|---|---|
| **N0 — Contacto** | Nombre, especie, edad aproximada, régimen | Contacto diplomático |
| **N1 — Rumores** | 1–2 rasgos como rumor (≈20% de error), estadísticas como rangos amplios | Embajada o mucho comercio |
| **N2 — Conocido** | Todos los rasgos reales; estadísticas en rango estrecho; ambición probable | Red de inteligencia ≥ 2 o ~2 años de relaciones |
| **N3 — Íntimo** | Estadísticas exactas, ambición real, salud, planes y apoyos | Agente en la corte o red de nivel 3 |

```
puntos_conocimiento = 1,0×embajada_permanente + 1,0×nivel_red_inteligencia (0–3)
                    + 0,5×años_de_contacto (máx. 1,5) + 1,5×agente_en_la_corte
N0: < 1 · N1: 1–2,4 · N2: 2,5–3,9 · N3: ≥ 4
```

El botón **"¿por qué nos odian/aprecian?"** muestra el desglose (especie, régimen, líder, agravios, intereses) **sólo con la información disponible**; las partes desconocidas aparecen como "?". Las estimaciones de probabilidad de operaciones de espionaje usan rangos según el nivel.

---

## 6. Operaciones de espionaje (fórmulas iniciales)

Cada operación se asigna a un agente (notable con intriga), con **presupuesto** y objetivo.

### Parámetros por operación

| Operación | `base_éxito` | Duración | Costo base (M cr)² | Tamaño (0–3) |
|---|---|---|---|---|
| Reunir información | 0,70 | 2–4 semanas | 0,3 | 1 |
| Soborno / chantaje | 0,50 | 1–2 meses | 1 | 1 |
| Propaganda / desinformación | 0,55 | 2–3 meses | 1,5 | 2 |
| Sabotaje | 0,45 | 1–3 meses | 2 | 2 |
| Apoyar a un candidato | 0,40 | 3–6 meses | 3 | 2 |
| Asesinato del líder | 0,25 | 2–5 meses | 6 | 3 |
| Golpe de Estado | 0,20 | 6–12 meses | 15 | 3 |

² Se multiplica por el tamaño del Estado objetivo (×0,5 a ×3).

### Probabilidad de éxito

```
p_éxito = clamp( (base_éxito
                 + 0,05 × (intriga_agente − 3)
                 + 0,10 × nivel_red_inteligencia
                 + 0,10 × (presupuesto/costo − 1)
                 + k_inestabilidad × inestabilidad_objetivo
                 − 0,10 × contrainteligencia_objetivo
                 − 0,10 × seguridad_personal_líder   [sólo asesinato]
                 ± modificadores de rasgos del objetivo ) × f_régimen, 0,02, 0,95 )
```

- `intriga_agente`: 0–6; `nivel_red`: 0–3; `contrainteligencia`: 0–3; `seguridad_personal_líder`: 0–3; `inestabilidad_objetivo`: 0–1.
- `k_inestabilidad`: golpe 0,35 · apoyo 0,15 · asesinato 0,10 · otros 0,10.
- **Rasgos del objetivo:** *Paranoico* −0,10 / *Confiado* +0,10 (soborno y chantaje); *Codicioso* o *Corrupto* +0,15 en soborno; *Astuto* −0,05.
- `f_régimen`:

| Operación | Democracia | Rep. oligárquica | Monarquía | Junta | Teocracia | Tecnocracia | Corporativismo |
|---|---|---|---|---|---|---|---|
| Golpe | 0,5 | 0,8 | 1,0 | 1,2 | 0,8 | 0,7 | 0,9 |
| Apoyo a candidato | 1,2 | 1,0 | 0,5 | 0,6 | 0,8 | 1,0 | 1,1 |
| Asesinato | 1,0 | 1,0 | 0,9 | 0,8 | 0,9 | 1,0 | 1,0 |
| Otras | 1,0 | 1,0 | 1,0 | 1,0 | 1,0 | 1,0 | 1,0 |

### Probabilidad de descubrimiento

```
p_descubrimiento = clamp( 0,05 + 0,10 × contrainteligencia_objetivo
                         + 0,05 × tamaño_operación
                         + k_régimen
                         − 0,04 × (intriga_agente − 3)
                         − 0,05 × nivel_red_inteligencia, 0,03, 0,90 )
```

- `k_régimen`: +0,05 en estados policiales (Junta, Teocracia, Monarquía autoritaria); +0,05 en democracias con prensa libre (descubrimiento posterior).
- Es **independiente** de `p_éxito` (decidido en 10).

### Consecuencias

| Resultado | Efecto |
|---|---|
| Éxito sin descubrir | El cambio ocurre; la atribución es sólo una sospecha (≈10% de que el objetivo sospeche de vos) |
| Éxito y descubierto | El cambio ocurre; **casus belli**, relación −40 con la facción, reputación "intrigante" −10 con los informados; el nuevo líder puede adquirir *Vengativo contra vos* |
| Fracaso sin descubrir | Se pierde el presupuesto; el agente puede quedar quemado (−1 de intriga o retirado) |
| Fracaso y descubierto | Agentes capturados (se pierden), relación −60, casus belli, reputación −15 |

- **Mártir:** tras un asesinato, `p_mártir` = 0,40 en democracias y teocracias, 0,25 en otros. Si se activa: el objetivo gana unidad (estabilidad +, aprobación del sucesor +, Ag +1 por 5 años).
- **Guerra civil:** un golpe exitoso en un régimen inestable puede detonar guerra civil (09).
- Los resultados se muestran como un **informe** con el porqué.

---

## 7. Efectos de un cambio de líder en la diplomacia

- Al cambiar el líder, la `postura` se recalcula con los rasgos nuevos y `w` del régimen.
- **Tratados vigentes:** el nuevo líder los **mantiene, cuestiona o repudia** según su Fi y su postura; en regímenes con `w` alto es más probable y rápido; en democracias requiere aprobación de grupos.
- **Opinión hacia el jugador:** se reinicia parcialmente (los agravios persisten con decaimiento según cultura, 10), pero el líder nuevo parte de la opinión base de la especie + sus rasgos.
- **Anuncio:** el jugador recibe una alerta de "cambio de líder" con los rasgos conocidos (según el nivel de conocimiento).

---

## 8. Interfaz

- **Ficha de líder** (propio y ajeno): rasgos (con "?" si no se conocen), estadísticas, ambición, apoyo y salud.
- **Panel de personalidad:** los 4 ejes y cómo se combinan con la especie y el régimen.
- **Árbol de sucesión:** candidatos, sus pesos y apoyos (según el conocimiento).
- **Planificador de operaciones:** probabilidades estimadas con rango, costo, duración y consecuencias posibles.
- **Línea del tiempo de líderes:** historial de gobernantes de cada facción (15).

---

## Conexiones

06 (notables, atributos) · 08 (perfiles culturales de especie, regímenes colmena) · 09 (régimen, grupos de interés, guerra civil, espionaje) · 10 (postura, operaciones, tratados) · 15 (crónica de líderes) · 16 (UI).

## Preguntas abiertas

- Calibrar pesos `w`, inercias, bonificaciones de rasgos y todas las probabilidades con simulaciones sin jugador.
- ¿Los rasgos adquiridos pueden volverse permanentes en el linaje (familias que "heredan" fama o infamia)?
- ¿Cuántos candidatos conviene mostrar a un jugador con poca inteligencia (¿sólo el favorito?)?
- Reglas concretas del régimen colmena/colectivo por especie (08).
- ¿Elecciones simuladas con **partidos** o sólo con grupos de interés? (09)
