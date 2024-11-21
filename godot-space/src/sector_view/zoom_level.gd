extends Node
class_name SectorZoomLevel

signal on_zoom_change(value: float)

@export var value: float = 1.0

func set_zoom_level(value: float) -> void:
    self.value = value
    self.on_zoom_change.emit(self.value)
