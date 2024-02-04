extends Node2D

@onready var main_gui = $MainGui

func _ready():
    main_gui.connect("on_click_sector_button", self.on_gui_click_sector)
    main_gui.connect("on_click_fleet_button", self.on_gui_click_fleet)
    
    main_gui.set_sectors([
        { "SectorInfo": {"label": "sector 0 0", "id": 0 } },
        { "SectorInfo": {"label": "sector 1 0", "id": 1 } },
        { "SectorInfo": {"label": "sector 1 1", "id": 2 } }
    ])
    main_gui.set_fleets([
        { "FleetInfo": {"label": "fleet 1", "id": 3 } },
        { "FleetInfo": {"label": "fleet 2", "id": 4 } }
    ])

func on_gui_click_sector(id):
    print("on_gui_click_sector ", id)

func on_gui_click_fleet(id):
    print("on_gui_click_fleet ", id)
