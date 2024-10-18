extends Object
class_name Utils

# Make the function static
static func find_nearest(target_position: Vector2, candidates: Array, is_valid: Callable) -> Node2D:
    var nearest_node: Node2D = null
    var nearest_distance: float = 1e10

    for child in candidates:
        if child is Node2D:
            if not is_valid.call(child):
                continue

            var distance = target_position.distance_to(child.position)

            # Update nearest node if the current one is closer
            if distance < nearest_distance:
                nearest_distance = distance
                nearest_node = child

    return nearest_node

static func remove_children(node: Node):
    for b in node.get_children():
        node.remove_child(b)
        b.queue_free()
