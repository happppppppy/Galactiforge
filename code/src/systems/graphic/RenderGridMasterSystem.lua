local RenderGridMasterSystem = class("RenderGridMasterSystem", System)

local function UpdateRenders(tg, grid_item, grid_inventory, grid_base)
  if grid_base ~= nil then
    love.graphics.draw(tg.tileset.image, tg.tileset.tiles[114], grid_item.x_render, grid_item.y_render, grid_base.render_angle, grid_item.grid_scale, grid_item.grid_scale,  tg.tileset.tile_width/2, tg.tileset.tile_height/2)
  end

  love.graphics.draw(tg.tileset.image, tg.active_frame, grid_item.x_render, grid_item.y_render, grid_item.t_render+grid_item.direction_rad, grid_item.grid_scale, grid_item.grid_scale,  tg.tileset.tile_width/2, tg.tileset.tile_height/2)
  
  if global_show_resource_count and grid_inventory ~= nil then   
    for i,v in pairs(grid_inventory.resource_output) do
      love.graphics.print(v.count, grid_item.x_render, grid_item.y_render, grid_item.t_render, grid_item.grid_scale, grid_item.grid_scale,tileset.tile_width/2, tileset.tile_height/2)
    end
    y_offset = 10
    for i,v in pairs(grid_inventory.resource_input) do
      love.graphics.print(v.count, grid_item.x_render, grid_item.y_render, grid_item.t_render, grid_item.grid_scale, grid_item.grid_scale,tileset.tile_width/2, tileset.tile_height/2-y_offset)
      y_offset = y_offset + 10
    end
  end
end

function RenderGridMasterSystem:draw()
  for index, value in pairs(self.targets.pool1) do
    local grid_item = value:get("GridItem")
    local grid_inventory = value:get("GridInventory")
    local tg = value:get("TileSetGrid")
    local grid_base = value:get("GridBaseGraphic")

    UpdateRenders(tg, grid_item, grid_inventory, grid_base)
  end

  for index, value in pairs(self.targets.pool2) do
    local tg = value:get("TileSetGrid")
    local physics = value:get("PositionPhysics")
    love.graphics.draw(tg.tileset.image, tg.active_frame, physics.body:getX(), physics.body:getY(), physics.body:getAngle(), 1, 1, tg.tileset.tile_width/2, tg.tileset.tile_height/2)
  end

end

function RenderGridMasterSystem:requires()
	return {pool1 = {"GridItem", "TileSetGrid"}, pool2 = {"PositionPhysics", "TileSetGrid"}}
end

return RenderGridMasterSystem