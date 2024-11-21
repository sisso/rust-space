extends Node2D

enum LogLevel{
    WARN,
    INFO,
    DEBUG,
    TRACE,
}


func _resolve_log_level() -> int:
    if self.log_level == LogLevel.WARN:
        return 0
    if self.log_level == LogLevel.DEBUG:
        return 2
    if self.log_level == LogLevel.TRACE:
        return 3
    return 1

@export var log_level: LogLevel
@export var game_api: GameApi
@export var gui: MainGui
@export var save_path: String
@export var load_prveious_game: bool = true
@export_category("data")
@export var prefabs_list: PrefabsList


func _ready():
    self.game_api.initialize(_resolve_log_level(), save_path)
    if self.load_prveious_game:
        self.game_api.continue_or_start()
    else:
        self.game_api.start_game()
    self.prefabs_list.set_list(self.game_api.list_shipyards_prefabs())
    self.gui.init(self.game_api)


func _process(delta):
    self.game_api.update(delta)
