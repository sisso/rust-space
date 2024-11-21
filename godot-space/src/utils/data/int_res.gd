extends Resource
class_name IntRes

signal on_change(old_value: int, new_value: int)

var value : int = 0:
  get:
    return value
  set (new_value):
    var old_value = self.value
    value = new_value
    self.on_change.emit(old_value, self.value)
