@tool
extends EditorScript

func new_spaceinf():
    return {
        "SpaceObjInfo": {
            "label": "",
            "pos": Vector2(0, 0),
            "is_fleet": false,
            "is_planet": false,
            "is_asteroid": false,
            "is_jump": false,
            "is_station": false,
            "is_star": false
        }
    }

func _run():    
    var objects = []
    
    var obj = new_spaceinf()
    obj["SpaceObjInfo"]["id"] = 0
    obj["SpaceObjInfo"]["label"] = "sum"
    obj["SpaceObjInfo"]["star"] = true
    objects.push_back(obj)
    
    obj = new_spaceinf()
    obj["SpaceObjInfo"]["id"] = 1
    obj["SpaceObjInfo"]["label"] = "fleet"
    obj["SpaceObjInfo"]["fleet"] = true
    obj["SpaceObjInfo"]["pos"] = Vector2(2.0, 0.0)
    objects.push_back(obj)

    obj = new_spaceinf()
    obj["SpaceObjInfo"]["id"] = 2
    obj["SpaceObjInfo"]["label"] = "planet"
    obj["SpaceObjInfo"]["planet"] = true
    obj["SpaceObjInfo"]["pos"] = Vector2(5.0, 0.0)
    obj["SpaceObjInfo"]["orbiting_pos"] = Vector2(0.0, 0.0)
    objects.push_back(obj)
    
    print("updating sector view objects")
    var scn = get_scene()
    scn.objects = objects
