class_name ObjInfoProvider extends Node

var game_api: GameApi
var id: int
var info: ObjExtendedInfo

func _init(game_api: GameApi, id: int):
    self.game_api = game_api
    self.id = id

func get_id() -> int:
    return self.id

func get_info() -> ObjExtendedInfo:
    if self.info == null:
        self.update()
    return self.info

func update() -> ObjExtendedInfo:
    self.info = self.game_api.describe_obj(id)
    return self.info
