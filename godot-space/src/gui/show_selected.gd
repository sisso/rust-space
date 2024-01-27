extends Node
class_name ShowSelected

@export var shipyard_popup_button: Button

var obj

signal on_click_show_shipyard_orders(obj)

func show_info(obj):
    self.obj = obj
    
    $label.text = str(obj.get_id()) + ": " + obj.get_label()

    var desc = ""
    desc += "kind: " + obj.get_kind() + "\n"
    desc += "command: " + obj.get_command() + "\n"
    desc += "action: " + obj.get_action() + "\n"

    if obj.get_cargo_size() > 0:
        desc += "\n"
        desc += "cargo:\n"
        
        for i in range(obj.get_cargo_size()):
            var c = obj.get_cargo(i)
            desc += "- " + c.get_label() + " ("+ str(c.get_id())+ "): " + str(c.get_amount())

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

    $desc.text = desc


func _on_shipyard_popup_button_pressed():
    emit_signal("on_click_show_shipyard_orders", self.obj)
