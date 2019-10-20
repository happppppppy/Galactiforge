local grid_functions = require("code/src/systems/grids/grid_functions")

local GridItemSystem = class("GridItemSystem", System)

function GridItemSystem:update(dt)
  for index, value in pairs(self.targets) do
    --Render updates
    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")
    local tileset = value:get("TileSetGrid")
    local grid_item = value:get("GridItem")

    local grid_base = value:get("GridBaseGraphic")

    if grid_base ~= nil then
      grid_base.render_angle = 0
    end

    grid_item.t_render = 0
    grid_item.x_render, grid_item.y_render, grid_item.t_pos_grid_physics, grid_item.x_pos_grid_physics, grid_item.y_pos_grid_physics = grid_functions:getGridPosition(grid_item.x, grid_item.y, grid_item.grid_scale, tileset, physics)
  end
end

function GridItemSystem:requires()
	return {"GridItem", "TileSetGrid"}
end

return GridItemSystem
