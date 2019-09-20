local MainKeySystem = class("MainKeySystem", System)

function MainKeySystem:fireEvent(event)
  if event.key == "escape" then
    love.quit()
  end

  if event.key == "kp+" then
    global_zoom_level = global_zoom_level + 0.1
  end

  if event.key == "kp-" then
    global_zoom_level = global_zoom_level - 0.1
  end

  if event.key == "i" then
    global_zoom_ship  = not global_zoom_ship
  end

  if event.key == "r" then
    global_show_resource_count = not global_show_resource_count
  end

  if event.key == "l" then
    lock_view_to_ship = not lock_view_to_ship
  end

  if event.key == "." then
    if global_component_index < #global_component_name_list then
      global_component_index = global_component_index + 1
    else
      global_component_index = 1
    end
  end

  if event.key == "," then
    if global_component_index > 1 then
      global_component_index = global_component_index - 1
    else
      global_component_index = #global_component_name_list
    end
  end

  if event.key == "]" then
    if global_component_direction_index < #global_component_directions then 
      global_component_direction_index = global_component_direction_index + 1
    else
      global_component_direction_index = 1
    end
  end

end


return MainKeySystem