@tool
extends Node2D
class_name MarkerGeneric

@export var color: Color = Color.WHITE
@export var radius: float = 1.0
@export var id: int = -1
@export var zoom_level: SectorZoomLevel

func _ready():
    queue_redraw()

func _draw():
    var radius = self.radius
    if self.zoom_level != null:
        radius *= 1.0 / self.zoom_level.value
    draw_circle(Vector2.ZERO, radius, color, true, false)
