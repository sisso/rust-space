#@tool
extends Node2D
class_name MarkerGeneric

@export var color: Color = Color.WHITE
@export var radius: float = 1.0
@export var id: int = -1
@export var zoom_level: SectorZoomLevel
@onready var _trail: Line2D = $trail

@export_category("trail")
@export var trail_caputre_time: float = 1.0
@export var max_points: int = 10
var _last_position: Vector2
var _positions: Array[Vector2] = []

func _ready():
    self.queue_redraw()
    self._last_position = self.global_position
    self._trail.default_color = self.color

func _draw():
    var radius = self.radius
    if self.zoom_level != null:
        radius *= 1.0 / self.zoom_level.value
    self.draw_circle(Vector2.ZERO, radius, color)

func _process(delta: float) -> void:
    self._update_tail()

func _update_tail() -> void:
    for i in range(0, self._positions.size()):
        var global_pos = self._positions[i]
        var local_pos = self.to_local(global_pos)
        self._trail.set_point_position(i ,local_pos)
 
func _on_trail_capture_time_timeout() -> void:
    self._last_position = self.global_position
    self._positions.push_front(self._last_position)
    
    if self._positions.size() < self.max_points:
        self._trail.add_point(self._last_position)
    else:
        self._positions.pop_back()
    
    self._update_tail()
