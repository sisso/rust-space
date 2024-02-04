class_name MainGui extends CanvasLayer

enum ScreenMode {
    NORMAL,
    BUILDING,
}

@export_category("containers")
@export var sectors_container: Container
@export var fleets_container: Container
@export var sectors_view: Node2D
@export var selected_object_container: ShowSelected
@export var building_panel: Container
@export var shipyard_orders_popup: ShipyardOrdersPopup

@export_category("top_bar")
@export var speed_label: Label

@export_category("state")
@export var screen_mode: ScreenMode = ScreenMode.NORMAL
@export var building_items: Array
@export var speed: float = 1.0
@export var previous_speed: float = 1.0

signal on_click_fleet_button(id)
signal on_click_sector_button(id)

signal on_click_object_at_sector_view(id)

signal on_click_start_building(selected_id, pos)

signal on_change_speed(new_speed)

signal on_set_shipyard_building_order(id, order_id)

func set_sectors(sectors):
    print("refresh_sectors ", sectors)
    for b in self.sectors_container.get_children():
        self.sectors_container.remove_child(b)
        b.queue_free()

    for obj in sectors:
        var btn = Button.new()
        btn.text = obj["label"]
        btn.pressed.connect(self._on_click_sector.bind(obj["id"]))
        self.sectors_container.add_child(btn)

func set_fleets(fleets):
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

func set_sector_objects(objects: Array):
    self.sectors_view.update_objects(objects)

func set_buildings(list: Array):
    self.building_items = list
    self._refresh_building_items()
    
func _refresh_building_items():
    var item_list = building_panel.get_node("item_list")
    item_list.clear()

    for i in self.building_items:
        item_list.add_item(i["label"])

func _on_click_sector(id):
    emit_signal("on_click_sector_button", id)

func _on_click_fleet(id):
    emit_signal("on_click_fleet_button", id)

func _set_panel(kind):
    self.fleets_container.visible = kind == "fleets"
    self.sectors_container.visible = kind == "sectors"
    self.selected_object_container.visible = kind == "selected"
    self.building_panel.visible = kind == "building"

func _on_click_fleets():
    self._set_panel("fleets")
    
func _on_click_sectors():
    self._set_panel("sectors")

func center_camera_at(id):
    self.sectors_view.center_camera_at(id)
    
func center_camera():
    self.sectors_view.center_camera()

func _on_sector_view_on_click_object(id):
    self.emit_signal("on_click_object_at_sector_view", id)

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
        emit_signal("on_click_start_building", selected_id, pos)        
        self.sectors_view.clear_cursor()
        self.set_building_panel_idle()

    self.sectors_view.set_cursor_building(on_click_building_callback)

func _on_button_cancel_building_plot_pressed():
    self.set_building_panel_idle()
    
func set_building_panel_idle():
    self.building_panel.get_node("item_list").show()
    self.building_panel.get_node("button_build").show()
    self.building_panel.get_node("button_cancel").hide()
    self.screen_mode = ScreenMode.NORMAL


func _on_speed_selector_item_selected(index):
    self.previous_speed = self.speed
    
    self.speed = 1.0
    match index:
        0: self.speed = 0.0
        1: self.speed = 0.1
        2: self.speed = 0.25
        3: self.speed = 0.5
        4: self.speed = 1.0
        5: self.speed = 2.0
        6: self.speed = 5.0
        7: self.speed = 10.0
    
    emit_signal("on_change_speed", self.speed)

func _unhandled_input(event):
    if event is InputEventKey:
        if Input.is_action_pressed("pause"):
            if self.speed == 0.0:
                self.speed = self.previous_speed
                emit_signal("on_change_speed", self.speed)
            else:
                self.previous_speed = self.speed
                self.speed = 0.0
                emit_signal("on_change_speed", self.speed)        

func _on_button_4_pressed():
    shipyard_orders_popup.show_popup(null)

func _on_selected_object_on_click_show_shipyard_orders(obj):
    shipyard_orders_popup.show_popup(obj)

func set_shipyard_prefabs(prefabs):
    self.shipyard_orders_popup.set_prefabs(prefabs)

func _on_shipyard_orders_popup_on_set_shipyard_building_order(id, order_id):
    emit_signal("on_set_shipyard_building_order", id, order_id)

func show_obj_details(obj_desc):
    self._set_panel("selected")
    self.selected_object_container.show_info(obj_desc)
