class_name MainGui extends CanvasLayer

enum ScreenMode {
    NORMAL,
    BUILDING,
}

@export_category("containers")
@export var sectors_container: Container
@export var fleets_container: Container
@export var stations_container: Container
@export var sectors_view: SectorView
@export var selected_object_container: ShowSelected
@export var building_panel: Container
@export var shipyard_orders_popup: ShipyardOrdersPopup

@export_category("top_bar")
@export var speed_label: Label
@export var speed_selector: OptionButton

@export_category("state")
@export var screen_mode: ScreenMode = ScreenMode.NORMAL
@export var building_items: Array
@export var speed_index: int = 4
@export var pause_previous_speed_index: int = 4
@export var selected_sector_id: int = -1
@export var selected_obj_id: int = -1

var game_api: GameApi
var speeds: Dictionary

func _ready():
    self.speeds = {
        0: 0.0,
        1: 0.1,
        2: 0.25,
        3: 0.5,
        4: 1.0,
        5: 2.0,
        6: 5.0,
        7: 10.0,
    }

func _get_speed_from_index(index) -> float:
    return self.speeds[int(index)]

func _get_speed_index(speed) -> int:
    for i in self.speeds.keys():
        if abs(self.speeds[i] - speed) < 0.0001:
            return i
    return -1

func _process(_delta):
    var _events = self.game_api.take_events()
    self._refresh_sector_view()
    self._refresh_time_label()

@warning_ignore("shadowed_variable")
func init(game_api):
    self.game_api = game_api

    var sectors = self.game_api.list_sectors()
    self.selected_sector_id = sectors[0]["id"]
    self.selected_obj_id = -1
    self._refresh_gui()

func _refresh_gui():
    self._set_sectors(self.game_api.list_sectors())
    self._set_fleets(self.game_api.list_fleets())
    self._set_buildings(self.game_api.list_buildings())
    self._set_stations(self.game_api.list_stations())
    self._refresh_sector_view()
    self._refresh_time_label()

func _refresh_time_label():
    self.speed_label.text = "Time: %0.2f" % self.game_api.get_total_time()

func _refresh_sector_view():
    if self.selected_obj_id != -1:
        var spos = self.game_api.resolve_space_position(self.selected_obj_id)
        if spos != null:
            self.selected_sector_id = spos["sector_id"]

    var objs_id = self.game_api.list_at_sector(self.selected_sector_id)

    var objects: Array[ObjExtendedInfo] = []
    for id in objs_id:
        var info = self.game_api.describe_obj(id)
        objects.push_back(info)

    self._set_sector_objects(objects)

func _set_sectors(sectors):
    print("refresh_sectors ", sectors)
    for b in self.sectors_container.get_children():
        self.sectors_container.remove_child(b)
        b.queue_free()

    for obj in sectors:
        var btn = Button.new()
        btn.text = obj["label"]
        btn.pressed.connect(self._on_click_sector.bind(obj["id"]))
        self.sectors_container.add_child(btn)

func _set_fleets(fleets):
    print("refresh_fleets ", fleets)
    for b in self.fleets_container.get_children():
        self.fleets_container.remove_child(b)
        b.queue_free()

    for obj in fleets:
        var id = obj["id"]
        var label = obj["label"]

        var btn = Button.new()
        btn.text = label
        btn.pressed.connect(self._on_click_fleet.bind(id))
        self.fleets_container.add_child(btn)

func _set_stations(stations: Array[LabelInfo]):
    print("refresh_staions ", stations)
    for b in self.stations_container.get_children():
        self.stations_container.remove_child(b)
        b.queue_free()

    for obj in stations:
        var id = obj.get_id()
        var label = obj.get_label()

        var btn = Button.new()
        btn.text = label
        btn.pressed.connect(self._on_click_station.bind(id))
        self.stations_container.add_child(btn)


func _set_sector_objects(objects: Array[ObjExtendedInfo]):
    self.sectors_view.update_objects(objects)

func _set_buildings(list: Array):
    self.building_items = list
    self._refresh_building_items()

func _refresh_building_items():
    var item_list = building_panel.get_node("item_list")
    item_list.clear()

    for i in self.building_items:
        item_list.add_item(i["label"])

func _on_click_sector(id):
    self.selected_sector_id = id
    self.selected_obj_id = -1
    self._refresh_sector_view()
    self._center_camera()

func _on_click_fleet(id):
    self._on_click_obj(id)

func _on_click_station(id):
    self._on_click_obj(id)

func _on_click_obj(id):
    self.selected_obj_id = id
    self._refresh_sector_view()
    self._show_obj_details(id)
    self._center_camera_at(self.selected_obj_id)

func _set_panel(kind):
    self.fleets_container.visible = kind == "fleets"
    self.sectors_container.visible = kind == "sectors"
    self.selected_object_container.visible = kind == "selected"
    self.building_panel.visible = kind == "building"
    self.stations_container.visible = kind == "stations"

func _on_click_fleets():
    self._set_panel("fleets")

func _on_click_stations():
    self._set_panel("stations")

func _on_click_sectors():
    self._set_panel("sectors")

func _center_camera_at(id: int):
    if not self.sectors_view.center_camera_at_obj(id):
        var ip = ObjInfoProvider.new(self.game_api, id)
        var info = ip.get_info()
        self.sectors_view.center_camera_at_pos(info.get_pos())


func _center_camera():
    self.sectors_view.center_camera()

func _on_sector_view_on_click_object(id):
    self.selected_obj_id = id
    self._show_obj_details(id)

func _on_button_building_pressed():
    self._set_panel("building")

func _on_button_build_plot_pressed():
    var selected = self.building_panel.get_node("item_list").get_selected_items()
    if selected.size() == 0:
        print("no item selected, skipping")
        return

    var index = selected[0]
    var selected_id = self.building_items[index]["id"]

    self.building_panel.get_node("item_list").hide()
    self.building_panel.get_node("button_build").hide()
    self.building_panel.get_node("button_cancel").show()
    self.screen_mode = ScreenMode.BUILDING

    var on_click_building_callback = func (pos):
        if self.selected_sector_id == -1:
            print("no sector_id selected, ignoring building")
            return
        print("sending new building site ", self.selected_sector_id, " ", selected_id, " ", pos)
        self.game_api.new_building_site(self.selected_sector_id, pos, selected_id)
        self.sectors_view.clear_cursor()
        self._set_building_panel_idle()

    self.sectors_view.set_cursor_building(on_click_building_callback)

func _on_button_cancel_building_plot_pressed():
    self._set_building_panel_idle()

func _set_building_panel_idle():
    self.building_panel.get_node("item_list").show()
    self.building_panel.get_node("button_build").show()
    self.building_panel.get_node("button_cancel").hide()
    self.screen_mode = ScreenMode.NORMAL
    self.sectors_view.clear_cursor()

func _on_speed_selector_item_selected(index):
    print("setting previous speed index to ", self.speed_index, " new index ", index)
    self.pause_previous_speed_index = self.speed_index
    self.speed_index = index

    var speed = self._get_speed_from_index(index)
    self.game_api.set_speed(speed)

func _update_speed_selector():
    self.speed_selector.selected = self.speed_index

func _unhandled_input(event):
    if event is InputEventKey:
        if Input.is_action_pressed("pause"):
            if self.speed_index != 0:
                self._on_speed_selector_item_selected(self._get_speed_index(0.0))
            else:
                self._on_speed_selector_item_selected(self.pause_previous_speed_index)
            self._update_speed_selector()

func _on_button_4_pressed():
    self.shipyard_orders_popup.show_popup(null)

func _on_selected_object_on_click_show_shipyard_orders(obj: ObjInfoProvider):
    self.shipyard_orders_popup.show_popup(obj)

func _on_shipyard_orders_popup_on_set_shipyard_building_order(id, order_id):
    self.selected_obj_id = id
    if order_id == null:
        self.game_api.cancel_shipyard_building_order(id)
    else:
        self.game_api.set_shipyard_building_order(id, order_id)

    self._show_obj_details(id)

func _show_obj_details(id: int):
    if id == -1:
        self.selected_object_container.clear_info()
    else:
        self._set_panel("selected")
        var provider = ObjInfoProvider.new(self.game_api, id)
        self.selected_object_container.show_info(provider)
