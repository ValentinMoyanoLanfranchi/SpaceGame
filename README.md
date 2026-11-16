# HELIARCA (nombre provisorio)

Un 4X espacial 2D por incrementos de tiempo, estilo **Aurora 4X** con economía mixta profunda, logística real y un mundo que sigue su curso. El jugador gobierna un Estado que parte de la **Tierra pre-espacial** y busca llegar al espacio y cruzar su primer punto de salto.

- **Núcleo de simulación:** Rust, determinista y dirigido por datos (JSON).
- **Interfaz:** Godot 4 + godot-rust (gdext).
- **Alcance actual:** el **MVP** = Fase 1 jugable (de la Tierra hasta cruzar el primer punto de salto).

## Documentación de diseño (GDD)
Está en [`Documentacion/`](Documentacion/). Puntos de entrada:

| Doc | Para qué |
|---|---|
| [00 Visión general](Documentacion/00_vision_general.md) | Qué es el juego y su índice |
| [17 Decisiones y roadmap](Documentacion/17_preguntas_abiertas_y_roadmap.md) | Decisiones tomadas, riesgos, preguntas abiertas |
| [21 Alcance del MVP y arquitectura](Documentacion/21_alcance_mvp_y_arquitectura.md) | Qué entra en el MVP y cómo se construye |
| [22 Datos iniciales del MVP](Documentacion/22_datos_iniciales_mvp.md) | Números concretos a cargar en JSON |
| [23 Plan de desarrollo](Documentacion/23_plan_de_desarrollo.md) | Cómo se ejecuta el MVP paso a paso (hitos, jerarquía, reglas) |

## Cómo se trabaja
El desarrollo está planificado en **issues de GitHub** con jerarquía `Hito → Épica → Historia → Tarea`. Leé [CONTRIBUTING.md](CONTRIBUTING.md) y el [plan de desarrollo](Documentacion/23_plan_de_desarrollo.md). Todo cambio entra por **Pull Request**; `main` está protegida (ver [SECURITY.md](SECURITY.md)).
