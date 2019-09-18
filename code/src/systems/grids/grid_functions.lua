local grid_functions = {}

function grid_functions:getRenderPositions(x_pos, y_pos, grid_scale, tg, physics, x_offset, y_offset)
  x_offset = x_offset or 0
  y_offset = y_offset or 0

  local offset_dist = math.sqrt((x_pos*tg.tileset.tile_width + x_offset)^2 + (y_pos*tg.tileset.tile_height + y_offset)^2)*grid_scale
  local offset_angle = math.atan2((y_pos*tg.tileset.tile_height + y_offset),(x_pos*tg.tileset.tile_width + x_offset))
  
  t_render = physics.body:getAngle()
  x_pos_render = physics.body:getX()+offset_dist*math.cos(t_render + offset_angle)
  y_pos_render = physics.body:getY()+offset_dist*math.sin(t_render + offset_angle)

  return t_render,x_pos_render,y_pos_render

end

function grid_functions:getResourceAvailable(grid_item)
  local resources_available = true
  for res_in_index,res_in_value in pairs(grid_item.resource_input) do
    if grid_item.resource_input[res_in_index].count <= 0 then
      resources_available = false
    end
  end

  return resources_available
end

function grid_functions:incrementResource(grid_item)
  for res_in_index,res_in_value in pairs(grid_item.resource_input) do
    grid_item.resource_input[res_in_index].count = grid_item.resource_input[res_in_index].count - 1
  end
  for res_out_index, res_out_value in pairs(grid_item.resource_output) do
    grid_item.resource_output[res_out_index].count = grid_item.resource_output[res_out_index].count + 1
  end
end

return grid_functions