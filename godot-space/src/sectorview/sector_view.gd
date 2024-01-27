@tool
extends Node2D

enum CursorMode {
    NORMAL,
    BUILDING,
}

@export_category("models prefab")
@export var prefab_marker: String
@export var prefab_orbit: String
@export var prefab_building_cursor: String

@export_category("models")
@onready var prefab_marker_scene = load(prefab_marker)
@onready var prefab_orbit_scene = load(prefab_orbit)
@onready var prefab_building_cursor_scene = load(prefab_building_cursor)

@export_category("colors")
@export var color_unknown: Color = Color.WHITE_SMOKE
@export var color_star: Color = Color.YELLOW
@export var color_planet: Color = Color.BLUE
@export var color_asteroid: Color = Color.DARK_BLUE
@export var color_fleet: Color = Color.RED
@export var color_station: Color = Color.BLUE_VIOLET
@export var color_jump: Color = Color.DARK_ORANGE
@export var color_orbit: Color = Color.DIM_GRAY
@export var orbit_color: Color = Color.PAPAYA_WHIP

@export_category("interaction")
@export var pixels_per_au: float = 100;
@export_range(0.0, 100.0, 1.0, "or_greater") var max_click_pixel_distance: float = 10.0

@export_category("state")
@export var objects = []
@export var cursor_mode: CursorMode = CursorMode.NORMAL
@export var cursor: Node2D = null

var cursor_callback = null

signal on_click_object(id)

func _ready():
    self.refresh_models()
    $distance_markers.distance_per_mark = self.pixels_per_au

func update_objects(objects):
    # print("updating objects ", objects)
    self.objects = objects
    self.refresh_models()

func refresh_models():
    # remove old nodes
    while $objects.get_child_count() > 0:
        var c = $objects.get_child(0)
        $objects.remove_child(c)
        c.queue_free()

    var orbits = []

    for obj in self.objects:
        var id = obj.get_id()
       
        var color = self.color_unknown
        if obj.is_fleet():
            color = self.color_fleet
        if obj.is_planet():
            color = self.color_planet
        if obj.is_asteroid():
            color = self.color_asteroid
        if obj.is_jump():
            color = self.color_jump
        if obj.is_station():
            color = self.color_station
        if obj.is_star():
            color = self.color_star
        if obj.is_orbiting():
            var parent_id = obj.get_orbit_parent_id()
            orbits.push_back([id, parent_id])            
        
        var marker = prefab_marker_scene.instantiate()        
        marker.position = obj.get_pos() * self.pixels_per_au
        marker.color = color
        marker.id = id
        
        $objects.add_child(marker)

    for orbit in orbits:
        var obj_marker = self._find_marker_by_id(orbit[0])
        var parent_marker = self._find_marker_by_id(orbit[1])
        
        var orbit_marker = self.prefab_orbit_scene.instantiate()
        orbit_marker.orbiting_obj = obj_marker
        orbit_marker.parent_obj = parent_marker
        orbit_marker.color = orbit_color
        $objects.add_child(orbit_marker)
        

func _find_marker_by_id(id):
    for c in $objects.get_children():
        if c is MarkerGeneric:            
            if c.id == id:
                return c
    return null

func _find_marker_by_position(pixel_position):
    var is_valid = func(node: Node2D):
        return node is MarkerGeneric
    
    var nearest = Utils.find_nearest(pixel_position, $objects.get_children(), is_valid)
    if nearest == null:
        return null        
      
    var distance = nearest.position.distance_to(pixel_position)
    if distance > self.max_click_pixel_distance:
        print("click ignored, too far away ", distance)
        return null

    #print("click at ", position, " found ", nearest.id)
    
    return nearest.id

func center_camera_at(id):
    for c in $objects.get_children():
        if c is MarkerGeneric:            
            if c.id == id:
                $camera.position = c.position
                return
    print("obj id ", id, " not found")
    
func center_camera():
    print("center camera")
    $camera.position = Vector2(0, 0)

func pixel_to_au(pixel_position):
    return pixel_position / self.pixels_per_au

func _on_camera_on_click_position(pixel_position):
    if self.cursor_mode == CursorMode.BUILDING:
        if self.cursor_callback != null:
            var au_pos = pixel_to_au(pixel_position)
            self.cursor_callback.call(au_pos)
    else:       
        var id = self._find_marker_by_position(pixel_position)
        if id != null:
            emit_signal("on_click_object", id)
        else:
            print("no object found at ", pixel_position)

func set_cursor_building(callback):
    print("set cursor buidling")
    self.cursor_mode = CursorMode.BUILDING
    self.cursor_callback = callback

    if self.cursor != null:
        self.cursor.queue_free()
        self.cursor = null

    self.cursor = self.prefab_building_cursor_scene.instantiate()
    $cursors.add_child(self.cursor)


func clear_cursor():
    self.cursor_mode = CursorMode.NORMAL
    self.cursor_callback = null
    if self.cursor != null:
        self.cursor.queue_free()
        self.cursor = null


func _process(delta):
    if self.cursor != null:
        var mouse_pos = get_viewport().get_mouse_position()
        var local_pos = get_viewport_transform().inverse() * mouse_pos
        # print("mouse pos: ", mouse_pos, " local pos: ", local_pos)
        self.cursor.position = local_pos
#        self.cursor.z_index = 1000
