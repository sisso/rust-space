@tool
extends Node2D

@export var objects: Array

@export var pos_scale: float = 200.0
@export var radius: float = 5.0
@export var color_unknown: Color = Color.WHITE_SMOKE
@export var color_star: Color = Color.YELLOW
@export var color_planet: Color = Color.BLUE
@export var color_asteroid: Color = Color.DARK_BLUE
@export var color_fleet: Color = Color.RED
@export var color_station: Color = Color.BLUE_VIOLET
@export var color_jump: Color = Color.DARK_ORANGE
@export var color_orbit: Color = Color.DIM_GRAY

func _ready():
    queue_redraw()
    
func update_objects(objects):
    self.objects = objects
    queue_redraw()
    
func _draw():
    pass
    #for obj in self.objects:
        #obj = obj["SpaceObjInfo"]
        #
        #var pos = obj["pos"]
        #pos *= self.pos_scale
        #
        #var color = self.color_unknown
        #if obj["is_fleet"]:
            #color = self.color_fleet
        #if obj["is_planet"]:
            #color = self.color_planet
        #if obj["is_asteroid"]:
            #color = self.color_asteroid
        #if obj["is_jump"]:
            #color = self.color_jump
        #if obj["is_station"]:
            #color = self.color_station
        #if obj["is_star"]:
            #color = self.color_star
        #
        #draw_arc(pos, self.radius, 0, 2*PI, 128, color, -1.0, false)
        #
        #if obj.has("orbiting_pos"):
            #var orbit_pos = obj["orbiting_pos"] * self.pos_scale
            #var distance = (pos - orbit_pos).length()
            #draw_arc(orbit_pos, distance, 0, 2*PI, 128, self.color_orbit, -1.0, false)
pass

#func _process(delta):
    #queue_redraw()
    
