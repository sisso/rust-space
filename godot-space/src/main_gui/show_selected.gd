class_name ShowSelected extends Node

@export var shipyard_popup_button: Button

var obj_info_provider: ObjInfoProvider

signal on_click_show_shipyard_orders(obj: ObjInfoProvider)

func show_info(obj_info_provider: ObjInfoProvider):
    self.obj_info_provider = obj_info_provider
    self._refresh()

func _refresh():
    var obj = self.obj_info_provider.get_info()
    if obj == null:
        print("ERROR: obj info provider return null obj")
        return

    $label.text = str(obj.get_id()) + ": " + obj.get_label()

    var desc = ""
    desc += "kind: " + obj.get_kind() + "\n"
    if obj.get_command() != "":
        desc += "command: " + obj.get_command() + "\n"
    if obj.get_action() != "":
        desc += "action: " + obj.get_action() + "\n"

    if obj.get_cargo_size() > 0:
        desc += "\n"
        desc += "cargo:\n"

        for i in range(obj.get_cargo_size()):
            var c = obj.get_cargo(i)
            desc += "- " + c.get_label() + " ("+ str(c.get_id())+ "): " + str(c.get_amount()) + "\n"

    if obj.get_resources().size() > 0:
        desc += "\n"
        desc += "extractable resources:\n"

        for i in obj.get_resources():
            desc += "- " + i.get_label() + " ("+ str(i.get_id())+ ")" + "\n"

    if obj.get_shipyard() != null:
        desc += "\n"
        desc += "shipyard: \n"

        if obj.get_shipyard().has_current_order():
            desc += "- producing " + str(obj.get_shipyard().get_current_order()) + "\n"
        else:
            desc += "- idle\n"

        if obj.get_shipyard().has_next_order():
            desc += "- next order " + str(obj.get_shipyard().get_next_order()) + "\n"

        self.shipyard_popup_button.show()
    else:
        self.shipyard_popup_button.hide()

    var requesting_wares = obj.get_requesting_wares()
    var providing_wares = obj.get_providing_wares()
    if obj.get_requesting_wares().size() > 0 || obj.get_providing_wares().size() > 0:
        desc += "\n"
        desc += "trading: \n"
        for ware in requesting_wares:
            desc += "- requesting " + ware.get_label() + "\n"
        for ware in providing_wares:
            desc += "- providing " + ware.get_label() + "\n"

    $desc.text = desc


func _process(delta: float):
    if self.obj_info_provider != null:
        self._refresh()
    else:
        $label.text = ""
        $desc.text = ""
        self.shipyard_popup_button.hide()

func _on_shipyard_popup_button_pressed():
    emit_signal("on_click_show_shipyard_orders", self.obj_info_provider)
