extends Node2D

enum LogLevel{
  WARN,
  INFO,
  DEBUG,
  TRACE,
}

func _resolve_log_level() -> int:
    if self.log_level == LogLevel.WARN:
        return 0
    if self.log_level == LogLevel.DEBUG:
        return 2
    if self.log_level == LogLevel.TRACE:
        return 3
    return 1

@export var log_level: LogLevel
@export var game_api: GameApi
@export var gui: MainGui
@export var save_path: String
@export var selected_sector_id: int = -1
@export var selected_obj_id: int = -1

func _ready():
    self.game_api.initialize(_resolve_log_level(), save_path)
    self.game_api.continue_or_start()

    # set selected sector
    var sectors = self.game_api.list_sectors()
    self.selected_sector_id = sectors[0]["id"]
    self.selected_obj_id = -1
    print("selected sector id ", self.selected_sector_id)
    
    self._refresh_gui()

func _process(delta):
    self.game_api.update(delta)
    var events = self.game_api.take_events()
    for e in events:
        pass
    self.refresh_sector_view()

func _refresh_gui():
    self.gui.set_sectors(self.game_api.list_sectors())
    self.gui.set_fleets(self.game_api.list_fleets())
    self.gui.set_buildings(self.game_api.list_buildings())
    self.gui.set_shipyard_prefabs(self.game_api.list_shipyards_prefabs())
    self.refresh_sector_view()

func refresh_sector_view():
    if self.selected_obj_id != -1:
        var spos = self.game_api.resolve_space_position(self.selected_obj_id)
        if spos != null:
            self.selected_sector_id = spos["sector_id"]
    
    var objs_id = self.game_api.list_at_sector(self.selected_sector_id)
    
    var objects = []
    for id in objs_id:
        var info = self.game_api.describe_obj(id)
        objects.push_back(info)
    
    self.gui.set_sector_objects(objects)

func _on_main_gui_on_click_fleet_button(id):
    self.selected_obj_id = id
    self.refresh_sector_view()
    self.gui.center_camera_at(self.selected_obj_id)

func _on_main_gui_on_click_sector_button(id):
    self.selected_sector_id = id
    self.selected_obj_id = -1
    self.refresh_sector_view()
    self.gui.center_camera()

func _on_main_gui_on_click_object_at_sector_view(id):
    self.selected_obj_id = id
    var desc = self.game_api.describe_obj(id)
    var obj_desc = self.game_api.describe_obj(id)
    self.gui.show_obj_details(obj_desc)

func _on_main_gui_on_click_start_building(selected_id, pos):
    if self.selected_sector_id == -1:
        print("no sector_id selected, ignoring building")
        return
    
    print("sending new building site ", self.selected_sector_id, " ", selected_id, " ", pos)
    self.game_api.new_building_site(self.selected_sector_id, pos, selected_id)


func _on_main_gui_on_change_speed(new_speed):
    self.game_api.set_speed(new_speed)

func _on_main_gui_on_set_shipyard_building_order(id, order_id):
    self.selected_obj_id = id
    if order_id == null:
        self.game_api.cancel_shipyard_building_order(id)
    else:
        self.game_api.set_shipyard_building_order(id, order_id)
    var obj_desc = self.game_api.describe_obj(id)
    self.gui.show_obj_details(obj_desc)
