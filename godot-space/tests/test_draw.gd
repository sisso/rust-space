@tool
extends Node2D


# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	queue_redraw()
	print("not working")

func _draw():
	draw_arc(Vector2(0, 0), 80, 0, 2*PI, 128, Color.RED, -1.0, true)
