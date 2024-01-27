@tool
extends Node2D
class_name MarkerGeneric

@export var color: Color = Color.WHITE
@export var radius: float = 1.0
@export var id: int = -1

func _ready():
    queue_redraw()
    
func _draw():
    draw_arc(Vector2.ZERO, self.radius, 0, 2*PI, 128, color, -1.0, false)
