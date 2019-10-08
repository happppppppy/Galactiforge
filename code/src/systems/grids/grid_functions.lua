local grid_functions = {}

function grid_functions:getGridPosition(x_pos, y_pos, grid_scale, tg, physics)
  local offset_dist = math.sqrt((x_pos*tg.tileset.tile_width)^2 + (y_pos*tg.tileset.tile_height)^2)*grid_scale
  local offset_angle = math.atan2((y_pos*tg.tileset.tile_height),(x_pos*tg.tileset.tile_width))
  
  x_pos_grid = offset_dist*math.cos(offset_angle)
  y_pos_grid = offset_dist*math.sin(offset_angle)
  
  t_pos_grid_physics =  physics.body:getAngle()
  x_pos_grid_physics = physics.body:getX()+offset_dist*math.cos(t_pos_grid_physics + offset_angle)
  y_pos_grid_physics =  physics.body:getY()+offset_dist*math.sin(t_pos_grid_physics + offset_angle)

  return x_pos_grid, y_pos_grid, t_pos_grid_physics, x_pos_grid_physics, y_pos_grid_physics
end

function grid_functions:getResourceAvailable(grid_inventory)
  local resources_found = {}
  local resources_available = false
  for resource_name,resource_details in pairs(grid_inventory.resources) do
    if resource_details.type == "stored" and resource_details.count > 0 then
      table.insert(resources_found, resource_name)
    end
  end

  if #resources_found > 0 then resources_available = true end

  return resources_found, resources_available
end

return grid_functions