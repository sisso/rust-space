#extends GuiPopup
extends Window
class_name ShipyardOrdersPopup

@export var buttons_container: Container

var prefabs = []
var obj_id: int

signal on_set_shipyard_building_order(id: int, order_id: int)

func set_prefabs(prefabs):
    self.prefabs = prefabs
    print("set shipyards prefabs ", prefabs)
    Utils.remove_children(self.buttons_container)
    for index in range(self.prefabs.size()):
        var btn = Button.new()
        btn.text = prefabs[index].get_label()
        btn.pressed.connect(self._on_click_prefab.bind(index))
        self.buttons_container.add_child(btn)

func show_popup(obj: ObjExtendedInfo):
    if obj == null:
        print("show popup with nil argument, ignoring")
        return
    self.obj_id = obj.get_id()
    self.show()

func _on_close_requested():
    self.hide()

func _on_click_prefab(index: int):
    emit_signal("on_set_shipyard_building_order", self.obj_id, self.prefabs[index].get_id())
    self.hide()

func _on_cancel_order_button_pressed():
    emit_signal("on_set_shipyard_building_order", self.obj_id, null)
    self.hide()
