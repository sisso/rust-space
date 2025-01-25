extends Node2D
class_name SectorView

enum CursorMode {
    NORMAL,
    BUILDING,
}

@export_category("models prefab")
@export var prefab_marker: PackedScene
@export var prefab_orbit: PackedScene
@export var prefab_building_cursor: PackedScene
@export var prefab_selected: PackedScene

@export_category("colors")
@export var color_unknown: Color = Color.WHITE_SMOKE
@export var color_star: Color = Color.YELLOW
@export var color_planet: Color = Color.BLUE
@export var color_asteroid: Color = Color.DARK_BLUE
@export var color_fleet: Color = Color.RED
@export var color_station: Color = Color.BLUE_VIOLET
@export var color_jump: Color = Color.DARK_ORANGE
@export var color_orbit: Color = Color.DIM_GRAY

@export_category("interaction")
@export var pixels_per_au: float = 100;
@export_range(0.0, 100.0, 1.0, "or_greater") var max_click_pixel_distance: float = 10.0

@export_category("state")
@export var objects_by_id = {}
@export var markers_by_id = {}
@export var cursor_mode: CursorMode = CursorMode.NORMAL
@export var zoom_level: SectorZoomLevel

@onready var objects_group: Node2D = %objects
@onready var distance_markers: Node2D = %distance_markers
@onready var camera: Camera2D = %camera
@onready var _cursors: Node2D = %cursors

var cursor: Node2D = null
var cursor_callback = null
var _selected_cursor: Node2D = null

signal on_click_object(id: int)

func _ready():
    self._clear_models()
    self.distance_markers.distance_per_mark = self.pixels_per_au

func update_objects(objects: Array[ObjExtendedInfo]):
    # print("updating objects ", objects)
    var new_orbits = {}
    var updated_objects = {}

    for obj in objects:
        var marker = self.markers_by_id.get(obj.get_id())
        if marker == null:
            marker = self._create_marker(obj)
            self.objects_by_id[obj.get_id()] = obj
            self.markers_by_id[obj.get_id()] = marker
            self.objects_group.add_child(marker)

            if obj.is_orbiting():
                var parent_id = obj.get_orbit_parent_id()
                new_orbits[obj.get_id()] = parent_id

        # update changes
        marker.position = self.game_pos_into_local(obj.get_pos())
        marker.zoom_level = self.zoom_level

        # mark object as added
        updated_objects[obj.get_id()] = true

    # update orbits
    for id in new_orbits:
        var obj_marker = self._find_marker_by_id(id)
        var parent_id = new_orbits[id]
        var parent_marker = self._find_marker_by_id(parent_id)

        var orbit_marker = self.prefab_orbit.instantiate()
        orbit_marker.orbiting_obj = obj_marker
        orbit_marker.parent_obj = parent_marker
        orbit_marker.color = color_orbit
        self.objects_group.add_child(orbit_marker)
        #self._orbits[id] = orbit_marker

     #check for removed objects
    for obj_id in self.markers_by_id:
        if !updated_objects.has(obj_id):
            self.markers_by_id[obj_id].queue_free()
            self.markers_by_id.erase(obj_id)
            self.objects_by_id.erase(obj_id)

            #if self._orbits.has(obj_id):
                #self._orbits[obj_id].queue_free()
                #self._orbits.erase(obj_id)

func _clear_models():
    self.objects_by_id = {}
    self.markers_by_id = {}
    #self._orbits = {}
    while self.objects_group.get_child_count() > 0:
        var c = self.objects_group.get_child(0)
        self.objects_group.remove_child(c)
        c.queue_free()

func _create_marker(obj: ObjExtendedInfo):
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

    var marker = self.prefab_marker.instantiate() as MarkerGeneric
    marker.color = color
    marker.id = id

    return marker

func _find_marker_by_id(id: int) -> MarkerGeneric:
    for c in self.objects_group.get_children():
        if c is MarkerGeneric:
            if c.id == id:
                return c
    return null

func _find_marker_by_position(pixel_position: Vector2) -> MarkerGeneric:
    var is_valid = func(node: Node2D):
        return node is MarkerGeneric

    var nearest = Utils.find_nearest(pixel_position, self.objects_group.get_children(), is_valid)
    if nearest == null:
        return null

    var distance = nearest.position.distance_to(pixel_position)
    if distance > self.max_click_pixel_distance:
        print("click ignored, too far away ", distance)
        return null

    #print("click at ", position, " found ", nearest.id)

    return nearest

# return true if object was found and camera moved, else if obj is not
# position on the map (like docked)
func center_camera_at_obj(id: int) -> bool:
    for c in self.objects_group.get_children():
        if c is MarkerGeneric:
            if c.id == id:
                self.camera.position = c.position
                return true
    return false

func center_camera_at_pos(pos: Vector2):
    self.camera.position = game_pos_into_local(pos)

func center_camera():
    self.camera.position = Vector2(0, 0)

func game_pos_into_local(pos: Vector2) -> Vector2:
    return pos * self.pixels_per_au

func screen_to_local(pixel_position):
    return pixel_position / self.pixels_per_au

func _on_camera_on_click_position(pixel_position):
    if self.cursor_mode == CursorMode.BUILDING:
        if self.cursor_callback != null:
            var au_pos = screen_to_local(pixel_position)
            self.cursor_callback.call(au_pos)
    else:
        var marker = self._find_marker_by_position(pixel_position)
        if marker != null:
            self.on_click_object.emit(marker.id)
            self._add_selected_cursor(marker)
        else:
            print("no object found at ", pixel_position)
            self._clear_selected_cursor()
            self.on_click_object.emit(-1)

func set_cursor_building(callback):
    self.cursor_mode = CursorMode.BUILDING
    self.cursor_callback = callback

    if self.cursor != null:
        self.cursor.queue_free()
        self.cursor = null

    self.cursor = self.prefab_building_cursor.instantiate()
    self._cursors.add_child(self.cursor)


func clear_cursor():
    self.cursor_mode = CursorMode.NORMAL
    self.cursor_callback = null
    if self.cursor != null:
        self.cursor.queue_free()
        self.cursor = null


func _process(delta) -> void:
    if self.cursor != null:
        var mouse_pos = get_viewport().get_mouse_position()
        var local_pos = get_viewport_transform().inverse() * mouse_pos
        self.cursor.position = local_pos

func _add_selected_cursor(marker: MarkerGeneric) -> void:
    self._clear_selected_cursor()
    var cursor = self.prefab_selected.instantiate() as SetPosition
    cursor.target = marker
    self._selected_cursor = cursor
    self._cursors.add_child(cursor)

func _clear_selected_cursor() -> void:
    if self._selected_cursor != null:
        self._selected_cursor.queue_free()
        self._selected_cursor = null
