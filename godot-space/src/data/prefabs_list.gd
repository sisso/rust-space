extends Resource
class_name PrefabsList

var list: Array[LabelInfo] = []

signal on_change(list: Array[LabelInfo])

func set_list(list: Array[LabelInfo]) -> void:
    self.list = list
    self.on_change.emit(self.list)
