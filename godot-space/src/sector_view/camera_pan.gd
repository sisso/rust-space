# https://github.com/Merlin1846/Panning-Camera-Plugin/blob/master/addons/Panning%20Camera/PanningCamera.gd
extends Camera2D
class_name PanningCamera

## The speed at which the camera pans when using the mouse.
@export_range(0.0, 10.0, 0.25, "or_greater") var sensitivity:float = 1.0
## The speed at which the camera zooms in and out.
@export var zoom_sensititvity:float = 0.5
## The speed at which the camera zooms in and out.
@export var zoom_mouse_sensititvity:float = 0.5
## The smallest the camera window can get.
@export var min_zoom: float = 0.25
## The largest the camera window can get.
@export var max_zoom: float = 5
## The speed at which the screen pans with the keyboard.
@export_range(0.0, 100.0, 1.0, "or_greater") var pan_speed: float = 10.0
## If true then the camera will pan when the mouse is at the edge of the screen.
@export var screen_edge_panning: bool = false
## The width of each edge pan area on the edge of the screen.
@export_range(0.0, 100.0, 1.0, "or_greater") var edge_pan_margin: float = 16.0
# The actions used for the various things.
@export_group("Inputs")
@export var pan_up: String = "camera_up"
@export var pan_down: String = "camera_down"
@export var pan_left: String = "camera_left"
@export var pan_right: String = "camera_right"
@export var zoom_in: String = "camera_zoom_in"
@export var zoom_out: String = "camera_zoom_out"
@export var mouse_pan: String = "mouse_left"
# @export var accelerated_panning: String = "accelerated_panning"

@export var zoom_level: SectorZoomLevel

signal on_click_position(position: Vector2)

var panning = false
var current_zoom: float = 1

func _ready():
    anchor_mode = Camera2D.ANCHOR_MODE_DRAG_CENTER
    self.current_zoom = self.zoom.x

func get_bounds():
    var size = get_viewport_rect().size / self.zoom
    var rect = Rect2(self.get_target_position() - size / 2, size)
    return rect

func _physics_process(_delta):
    # Key board controls.
    position += (Input.get_vector(pan_left, pan_right, pan_up, pan_down)*pan_speed)/zoom

    # Screen edge panning
    if screen_edge_panning && !Input.is_action_pressed(mouse_pan):
        if get_local_mouse_position().x >= ((get_window().get_size().x-(get_window().get_size().x/2))-edge_pan_margin)/zoom.x:
            position.x += pan_speed/zoom.x
        elif get_local_mouse_position().x <= (edge_pan_margin-(get_window().get_size().x/2))/zoom.x:
            position.x -= pan_speed/zoom.x
        if get_local_mouse_position().y >= ((get_window().get_size().y-(get_window().get_size().y/2))-edge_pan_margin)/zoom.y:
            position.y += pan_speed/zoom.x
        elif get_local_mouse_position().y <= (edge_pan_margin-(get_window().get_size().y/2))/zoom.y:
            position.y -= pan_speed/zoom.x

func _update_zoom(change: float) -> void:
    self.current_zoom += change * self.current_zoom
    self.zoom = Vector2(self.current_zoom, self.current_zoom)
    self.zoom_level.set_zoom_level(self.current_zoom)

func _process(delta):
    if Input.is_action_pressed(zoom_in) and self.current_zoom < self.max_zoom:
        self._update_zoom(self.zoom_sensititvity * delta)
    elif Input.is_action_pressed(zoom_out) and self.current_zoom > self.min_zoom:
        self._update_zoom(-self.zoom_sensititvity * delta)
    if Input.is_action_just_pressed(zoom_in) and self.current_zoom < self.max_zoom:
        self._update_zoom(self.zoom_mouse_sensititvity)
    if Input.is_action_just_pressed(zoom_out) and self.current_zoom > self.min_zoom:
        self._update_zoom(-self.zoom_mouse_sensititvity)

func _unhandled_input(event):
    # Mouse panning
    if event is InputEventMouseMotion:
        if Input.is_action_pressed(self.mouse_pan):
            if not self.panning:
                self.panning = true
            self.global_position -= (event.relative/self.zoom)*self.sensitivity

    if event is InputEventMouseButton:
        if Input.is_action_just_released(self.mouse_pan):
            if self.panning:
                self.panning = false
            else:
                var local_pos = get_viewport_transform().affine_inverse() * event.position
                emit_signal("on_click_position", local_pos)
