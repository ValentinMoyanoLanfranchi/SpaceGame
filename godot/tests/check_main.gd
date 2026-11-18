extends SceneTree
# Chequeo headless: carga Main.tscn, espera un frame y comprueba que la etiqueta muestra el texto del
# núcleo. Además de compararla con SimHandle.hello(), la contrasta con valores que NO salen de
# hello(): la fecha inicial del núcleo y la versión de `[workspace.package]` del Cargo.toml raíz
# (la que reporta heliarca-sim), para que un cambio en la cola de `hello_text` no pase inadvertido.
# Uso: godot --headless --path godot --script res://tests/check_main.gd
# Sale con código 1 si algo no coincide.

const FECHA_INICIAL := "Año 1, día 1, 00:00:00"

func _initialize() -> void:
	var main: Node = load("res://Main.tscn").instantiate()
	root.add_child(main)
	await process_frame
	var hello: String = SimHandle.new().hello()
	var actual: String = main.get_node("Label").text
	var version := _version_del_workspace()
	var esperado := "Heliarca (sim %s): %s" % [version, FECHA_INICIAL]
	if version.is_empty():
		_fallar("no se pudo leer la versión de [workspace.package] en Cargo.toml")
	elif actual != hello:
		_fallar("la etiqueta ('%s') no coincide con SimHandle.hello() ('%s')" % [actual, hello])
	elif actual != esperado:
		_fallar("etiqueta='%s' esperado='%s' (versión y fecha inicial del núcleo)" % [actual, esperado])
	else:
		print("OK: la etiqueta muestra '%s'" % actual)
		quit(0)

# Versión del paquete del workspace (Cargo.toml raíz, un nivel sobre `godot/`).
func _version_del_workspace() -> String:
	var ruta := ProjectSettings.globalize_path("res://").path_join("../Cargo.toml")
	var archivo := FileAccess.open(ruta, FileAccess.READ)
	if archivo == null:
		return ""
	var re := RegEx.create_from_string('(?m)^version = "([^"]+)"')
	var m := re.search(archivo.get_as_text())
	return "" if m == null else m.get_string(1)

func _fallar(mensaje: String) -> void:
	printerr("FALLO: " + mensaje)
	quit(1)