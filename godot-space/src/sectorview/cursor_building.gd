@tool
extends Node2D
class_name CursorBuilding

@export var color: Color = Color.WHITE
@export var radius: float = 1.0

func _ready():
    queue_redraw()

func _draw():
    draw_rect(Rect2(-self.radius, -self.radius, self.radius, self.radius), self.color, false, 2)
