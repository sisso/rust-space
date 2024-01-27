@tool
extends Node2D

@export var color: Color = Color.WHITE
@export var distance_per_mark: float = 1.0
@export var total_marks: int = 10

func _ready():
    queue_redraw()

func _draw():
    var radius = self.distance_per_mark
    for i in range(self.total_marks):    
        draw_arc(Vector2.ZERO, radius, 0, 2*PI, 128, color, -1.0, false)
        radius += distance_per_mark
