class_name ObjInfoProvider extends Node

var game_api
var id

func init(game_api, id):
    self.game_api = game_api
    self.id = id

func get_id():
    return self.id

func get_info():
    return self.game_api.describe_obj(id)
