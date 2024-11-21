extends Node

func _ready():
    pass # Replace with function body.

func _process(delta):
    pass

func initialize(log_level, save_path):
    pass

func continue_or_start():
    pass

func list_sectors():
    return [
        {
            "id": 0,
            "label": "sector 1"
        }
    ]

func list_fleets():
    return []

func list_buildings():
    return []

func list_shipyards_prefabs():
    return []

func list_at_sector(sector_id):
    return []

func get_total_time():
    return 0.0

func update(delta):
    pass

func take_events():
    return []

func set_speed(speed):
    pass
