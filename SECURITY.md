# Política de seguridad

## Reportar una vulnerabilidad
No abras un issue público. Usá el **reporte privado de vulnerabilidades** de GitHub:
<https://github.com/ValentinMoyanoLanfranchi/SpaceGame/security/advisories/new>

## Medidas del repositorio
- `main` está protegida por un *ruleset*: sin push directo, sin force-push, sin borrado, historial lineal y **todo cambio entra por Pull Request** (squash) con el check `validate` en verde y conversaciones resueltas. Se aplica también a los administradores.
- *Secret scanning* y *push protection* activados.
- Alertas y actualizaciones de seguridad de Dependabot activadas.
- El token de GitHub Actions es de **solo lectura** por defecto; las acciones se fijan por SHA y se actualizan con Dependabot.
- Los PR de colaboradores externos requieren aprobación manual antes de correr workflows.

## Reglas para el código
- Nunca versionar credenciales, tokens ni claves (ni siquiera de prueba reales).
- El núcleo de simulación (`heliarca-sim`) no hace red ni accede al sistema de archivos salvo por la API de guardado; `unsafe` está prohibido allí.
- Los archivos de datos y de guardado se tratan como **entrada no confiable**: se validan al cargar y los errores no deben provocar `panic`.
