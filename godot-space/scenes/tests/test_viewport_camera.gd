extends Node2D

@export var move_speed: float = 400.0
@export var zoom_speed: float = 1

# To be able to auto size into all visible objects we need to know
# the size of screen, position of all objects in scene
# - objects posistions should not be scaled
func center_objects():
    # get screen size 
    print("center objects");
   
    #var camera_size = get_node("../Panel").size
    #print("found ", camera_size)

func _ready():
    self.center_objects()

func _process(delta):
    if Input.is_action_pressed("camera_right"):
        self.position.x += self.move_speed * delta
    if Input.is_action_pressed("camera_left"):
        self.position.x -= self.move_speed * delta
    if Input.is_action_pressed("camera_up"):
        self.position.y -= self.move_speed * delta
    if Input.is_action_pressed("camera_down"):
        self.position.y += self.move_speed * delta
    if Input.is_action_pressed("camera_zoom_in"):
        var change = self.zoom_speed * delta
        self.scale *= 1.0 + change
    if Input.is_action_pressed("camera_zoom_out"):
        var change = self.zoom_speed * delta
        self.scale *= 1.0 - change
