# 06 — Personajes (notables)

> v0.3 · Los **rasgos, la personalidad de 4 ejes, los regímenes, la sucesión y los niveles de conocimiento de líderes** están definidos en `20_lideres_regimenes_y_rasgos.md`; este documento define el marco general de los notables. En el MVP se simulan líderes, gobernadores, directores y científicos (21, 22).

## Rol de los personajes

El jugador **no controla un avatar ni individuos**: gobierna un Estado. Los personajes son **notables**: recursos de gestión con voluntad propia, que el jugador nombra, asigna y a veces no controla. Son la forma en que el Estado "se hace persona".

Se simulan como individuos sólo los notables; la población general es agregada (05).

### Tipos de notable
- **Líder** de cada facción (jefe de Estado), incluido el del jugador (ver abajo y 20).
- **Gobernadores** de colonia o sistema.
- **Almirantes y comandantes** de flotas; **generales** de fuerzas terrestres (Fase 2).
- **Directores** de empresas estatales y **magnates** de empresas privadas (incluidas las aeroespaciales, 03, 07).
- **Científicos y jefes de proyecto** (13).
- **Diplomáticos y embajadores.**
- **Espías y jefes de inteligencia.**
- **Candidatos y líderes de grupos de interés, predicadores, caudillos, sindicalistas.**
- **Piratas, señores de la guerra, contrabandistas** (del mundo, no del jugador; desde la Fase 2).
- Cualquier individuo que se vuelva relevante durante la partida (héroe de una batalla, magnate emergente).

---

## Atributos

### Aptitudes
Rangos según especie (08): inteligencia, carisma, voluntad, aptitud técnica, aptitud táctica, etc.

### Habilidades (por uso y entrenamiento)
- **Mando:** táctica naval, táctica terrestre, logística militar, liderazgo.
- **Gestión:** administración, economía, industria, comercio.
- **Conocimiento:** **investigación por campo** (escala 0–6 en cada uno de los 8 campos tecnológicos, 13), ingeniería, xenología.
- **Sociales:** diplomacia, intriga, propaganda, oratoria.
- **Operativas:** pilotaje y maniobra de flotas, reconocimiento, relevamiento.

Se mejoran **usándolas** y con entrenamiento. Los rasgos modifican techo y velocidad. Los cargos aplican **bonificaciones** a la unidad que dirigen (una flota con un buen almirante rinde más; un buen gobernador reduce corrupción).

**Científicos y laboratorios (13):** la investigación se hace en laboratorios con científicos asignados; la habilidad de investigación de cada científico en el campo del laboratorio entra en la fórmula de producción de PI (`PI_mes = 10 × (0,7 + 0,1 × hab)` por laboratorio, con rendimientos decrecientes 0,93ⁿ).

### Rasgos
Sistema de **15 pares de rasgos opuestos (30 rasgos)** definido en 20 (p. ej. valiente/cobarde, ambicioso/conformista, paranoico/confiado, xenófobo/abierto, codicioso/austero, devoto/escéptico, genio/torpe, etc.), más rasgos **adquiridos** (veterano, traumatizado, corrupto, celebridad, fanático). Los rasgos desplazan los ejes de personalidad, las estadísticas y las probabilidades de eventos.

### Personalidad de 4 ejes (20)
Cada notable relevante tiene cuatro ejes (−3 a +3): **Agresividad (Ag), Apertura (Ap), Fiabilidad (Fi) y Codicia (Co)**. Resultan de la **especie** (disposición base, 08), la **cultura**, los **rasgos** y la **experiencia**. Alimentan decisiones, diplomacia (10) y primer contacto (02).

### Valores e identidad
Cultura, religión/ideología, lealtades (a la facción, a un grupo, a un líder), opiniones políticas.

### Relaciones y memoria
Opinión de otros notables; familia, amistades, rivalidades, mentores; **memoria de eventos** ("me salvó la vida", "me destituyó injustamente").

### Ambiciones
Riqueza, poder, venganza, conocimiento, fe, proteger a la familia, fama. Impulsan sus decisiones (01).

---

## Líderes de facción (modelo EU4 + CK3)

Todas las facciones —el jugador y las demás— tienen un **líder** cuyos atributos moldean su comportamiento (detalle completo en 20):

- **Estadísticas de gobierno** (estilo EU4): administración, diplomacia, economía, militar. Aportan modificadores al Estado.
- **Rasgos, personalidad y ambiciones** (estilo CK3): definen su actitud hacia otras facciones y hacia su propio pueblo (10).
- **Peso del líder (w):** depende del régimen (alto en autocracias, bajo en democracias). Hay **7 regímenes**, cada uno con su peso `w` e **inercia** de cambio (09, 20).
- **Edad, salud y herederos:** el líder envejece, enferma y muere; su sucesión sigue las reglas del régimen: **pool de 3–5 candidatos** y mecanismo propio (herencia, elección, golpe, designación, colmena, etc.).

### El líder del jugador
No es un avatar, pero **sus cualidades modifican al Estado** (un buen administrador mejora la capacidad administrativa; un líder paranoico agrava el malestar). El jugador no lo "controla", pero puede influir en quién gobierna reformando el régimen, apoyando candidatos, o enfrentando crisis de sucesión.

### Niveles de conocimiento de un líder ajeno (N0–N3)
Qué ve el jugador de la ficha de un líder extranjero depende de lo que **sabe** (20): **N0** nada (sólo cargo), **N1** nombre y rasgo visible, **N2** rasgos y ejes aproximados, **N3** ficha completa. Se mejora con contacto diplomático, embajadas, comercio y **espionaje** (p_éxito y p_descubrimiento, 20, 10).

### Cambio de líder
Por muerte, vejez, abdicación, elecciones, golpe, revolución o asesinato. Cuando ocurre, las políticas y relaciones se reajustan según el nuevo líder (10).

---

## Salud y vida (simplificado)

Sin modelo de partes del cuerpo. Cada notable tiene: **edad**, estado de salud general, enfermedades, heridas (si participa en combate), estrés/moral y **longevidad** según especie. Puede **morir** (vejez, enfermedad, combate, atentado, accidente, ejecución) o **retirarse**. La medicina y la tecnología pueden extender la vida.

## Estado de ánimo y lealtad

`lealtad = f(trato, ideología vs. leyes vigentes, recompensas, éxitos, rasgos, intervención del jugador)`

- Lealtad baja → ineficiencia, corrupción, conspiración, deserción (con su flota o su colonia), golpe de Estado.
- Lealtad alta → **inspiraciones**: avances de investigación, victorias, reformas exitosas.

---

## Control del jugador

- **Nombrar y destituir**: asignar cargos (gobernador, almirante, director, embajador, jefe de laboratorio).
- **Directivas:** prioridades, presupuesto, doctrina, prohibiciones.
- Los notables **tienen voluntad**: pueden discutir, negarse, conspirar o traicionar.
- **Costo de intervenir:** corregir una y otra vez a un delegado erosiona su lealtad (02).
- **Aprendizaje:** un notable que sirve bien ascende, gana experiencia y puede volverse políticamente relevante (o peligroso).

---

## Notables del mundo

- Generados en la historia previa (14) y **emergentes** durante la partida.
- Envejecen, tienen hijos, mueren; dejan herencias y rencores.
- Son **figuras históricas** consultables en la crónica (15).
- **Ascenso emergente:** un contrabandista exitoso puede volverse señor de la guerra; un empresario, fundar una megacorporación o un movimiento político.

## Reclutamiento

Formar notables (academias, educación estatal o privada), contratar extranjeros, reclutar desertores, mercenarios, capturar y "convertir" (según leyes de la facción).

## Carrera

Rangos, títulos, cargos, experiencia, reputación. Un buen almirante gana fama → puede volverse políticamente peligroso.

## Muerte y legado

Funerales según cultura, monumentos, naves con su nombre, herencias, rencores que persisten, objetos que pasan de mano en mano (07, 15).

## Límite de escala

Cantidad máxima de notables simulados en detalle (por rendimiento). Los menos relevantes pasan a estado **latente**. En el MVP el conjunto es pequeño (líder, algunos gobernadores, directores de las 3–4 empresas, científicos de laboratorio; 22).

## Conexiones

01 (agentes) · 02 (delegación, primer contacto) · 05 (gobernadores) · 07 (diseñadores, directores) · 08 (disposición base por especie) · 09 (régimen, intriga, sucesión) · 10 (líderes y diplomacia) · 11 (almirantes, doctrinas) · 13 (científicos y laboratorios) · 15 (figuras históricas) · 20 (rasgos, ejes, regímenes, sucesión, conocimiento).

## Preguntas abiertas

- ¿Herencia genética de rasgos? (Probablemente sólo cultural en el MVP.)
- ¿Cuántos notables simultáneos por facción (jugador e IA)? Calibrar con prototipos.
- Lista final de habilidades (propuesta: ~15–20) y su efecto numérico.
