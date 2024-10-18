class_name ObjInfoProvider extends Node

var game_api: GameApi
var id: int

func init(game_api: GameApi, id: int):
    self.game_api = game_api
    self.id = id

func get_id() -> int:
    return self.id

func get_info() -> ObjExtendedInfo:
    return self.game_api.describe_obj(id)
