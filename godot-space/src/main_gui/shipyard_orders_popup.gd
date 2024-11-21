#extends GuiPopup
extends Window
class_name ShipyardOrdersPopup

@export var buttons_container: Container
@export var prefabs_list: PrefabsList

var info_provider: ObjInfoProvider
var buttons_created = false;

signal on_set_shipyard_building_order(id: int, order_id: int)

func assert_buttons_created():
    if self.buttons_created:
        return

    Utils.remove_children(self.buttons_container)
    for index in range(self.prefabs_list.list.size()):
        var btn = Button.new()
        btn.text = self.prefabs_list.list[index].get_label()
        btn.pressed.connect(self._on_click_prefab.bind(index))
        self.buttons_container.add_child(btn)

    self.buttons_created = true

func show_popup(ip: ObjInfoProvider):
    if ip == null:
        print("show popup with nil argument, ignoring")
        return

    self.assert_buttons_created()

    self.info_provider = ip
    self.show()

func _on_close_requested():
    self.hide()

func _on_click_prefab(index: int):
    self.on_set_shipyard_building_order.emit(self.info_provider.get_id(), self.prefabs_list.list[index].get_id())
    self.hide()

func _on_cancel_order_button_pressed():
    self.on_set_shipyard_building_order.emit(self.info_provider.get_id(), null)
    self.hide()
