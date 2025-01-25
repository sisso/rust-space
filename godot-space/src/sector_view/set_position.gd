extends Node2D
class_name SetPosition

@export var target: Node2D

func _process(_delta: float) -> void:
    if self.target == null:
        self.queue_free()
    
    self.global_position = self.target.global_position
