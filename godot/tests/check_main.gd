extends SceneTree
# Chequeo headless: carga Main.tscn, espera un frame y compara la etiqueta con SimHandle.hello().
# Uso: godot --headless --path godot --script res://tests/check_main.gd
# Sale con código 1 si la etiqueta no muestra el texto del núcleo.

func _initialize() -> void:
	var main: Node = load("res://Main.tscn").instantiate()
	root.add_child(main)
	await process_frame
	var expected: String = SimHandle.new().hello()
	var actual: String = main.get_node("Label").text
	if actual != expected or not expected.begins_with("Heliarca (sim "):
		printerr("FALLO: etiqueta='%s' esperado='%s'" % [actual, expected])
		quit(1)
		return
	print("OK: la etiqueta muestra '%s'" % actual)
	quit(0)
