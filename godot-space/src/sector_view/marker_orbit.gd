extends Node2D
class_name MarkerOrbit

# self should belong to the origin of the parent object being orbit

@export var color: Color = Color.WHITE
@export var orbiting_obj: Node2D
@export var parent_obj: Node2D

func _draw():
    self.position = parent_obj.position
    var distance = (self.position - orbiting_obj.position).length()
    draw_arc(Vector2(0, 0), distance, 0, 2*PI, 128, color, -1.0, false)

func _process(delta: float) -> void:
    if self.orbiting_obj == null || self.parent_obj == null:
        self.queue_free()
    else:
        self.queue_redraw()
