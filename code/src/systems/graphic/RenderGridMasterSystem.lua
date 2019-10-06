local RenderGridMasterSystem = class("RenderGridMasterSystem", System)

local function UpdateRenders(tg, grid_item, grid_inventory, grid_base, physics)
  love.graphics.push()
  love.graphics.translate(physics.body:getX(),physics.body:getY())
  love.graphics.rotate(physics.body:getAngle())
  
  if grid_base ~= nil then
    love.graphics.draw(tg.tileset.image, tg.tileset.tiles[114], grid_item.x_render, grid_item.y_render, grid_base.render_angle, grid_item.grid_scale, grid_item.grid_scale,  tg.tileset.tile_width/2, tg.tileset.tile_height/2)
  end

  love.graphics.draw(tg.tileset.image, tg.active_frame, grid_item.x_render, grid_item.y_render, grid_item.t_render+grid_item.direction_rad, grid_item.grid_scale, grid_item.grid_scale,  tg.tileset.tile_width/2, tg.tileset.tile_height/2)
  
  if global_show_resource_count and grid_inventory ~= nil then   
    local y_offset = 0
    for i,v in pairs(grid_inventory.resources) do
      love.graphics.print(string.format("%s %02.0f", i,v.count), grid_item.x_render, grid_item.y_render, grid_item.t_render, grid_item.grid_scale, grid_item.grid_scale,tileset.tile_width/2, tileset.tile_height/2-y_offset)
      y_offset = y_offset + 10
    end
  end

  love.graphics.pop()
end

function RenderGridMasterSystem:draw()
  for index, value in pairs(self.targets.pool1) do
    local grid_item = value:get("GridItem")
    local grid_inventory = value:get("GridInventory")
    local tg = value:get("TileSetGrid")
    local grid_base = value:get("GridBaseGraphic")
    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")

    UpdateRenders(tg, grid_item, grid_inventory, grid_base, physics)
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