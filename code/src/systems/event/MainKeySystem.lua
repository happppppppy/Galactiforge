local json = require "code/lib/json"

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

  if event.key == "b" then
    global_build_mode = not global_build_mode
    if global_build_mode then
      global_zoom_level = 2
    else
      global_zoom_level = 1
    end
  end

  if event.key == "kp*" then
    if global_build_mode then
      for _,parent in pairs(global_player_ship) do
        local master_grid = parent:get("GridMaster")
        local ship = {new_ship = {}}
        if master_grid ~= nil then
          for _,v in pairs(master_grid.grid_items) do
            for _,b in pairs(v) do
              if b ~= 0 then
                b.active_resource = nil
                local proc = b:get("GridProcessor")
                if proc ~= nil then
                  for r_n,r in pairs(proc.resource_produced) do
                    if r.active then 
                      b.active_resource = r_n
                      break 
                    end
                  end
                end
                local item_table = {type = b.type, x = b.x, y = b.y, direction = b.direction, active_resource = b.active_resource}
                table.insert(ship.new_ship, item_table)
              end
            end
          end
          local json_data = json.encode(ship)
          local file = io.open("ship_save.json", "w")
          file:write(json_data)
          file:close()
        end
      end
    end
  end

end


return MainKeySystem