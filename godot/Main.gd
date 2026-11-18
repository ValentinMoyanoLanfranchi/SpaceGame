extends Control
# Muestra en la etiqueta el texto que arma el núcleo (heliarca-sim) a través de SimHandle.
# Sin lógica de juego: sólo pide el texto y lo muestra.

func _ready() -> void:
	var sim := SimHandle.new()
	var text: String = sim.hello()
	$Label.text = text
	print(text)
