local grid_functions = require("code/src/systems/grids/grid_functions")

local GridItemSystem = class("GridItemSystem", System)

function GridItemSystem:onRemoveEntity(entity)
  local grid_item = entity:get("GridItem")
  local parent = entity:getParent()
  local grid_master = parent:get("GridMaster")

  grid_master.grid_status[grid_master.grid_specs.allowed_grid.grid_origin.y - grid_item.y][grid_master.grid_specs.allowed_grid.grid_origin.x - grid_item.x] = 0
end

function GridItemSystem:update(dt)
  for index, value in pairs(self.targets) do
    --Render updates
    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")
    local tileset = value:get("TileSetGrid")
    local grid_item = value:get("GridItem")

    local grid_base = value:get("GridBaseGraphic")

    if grid_base ~= nil then
      grid_base.render_angle = physics.body:getAngle()
    end

    grid_item.t_render, grid_item.x_render, grid_item.y_render = grid_functions:getRenderPositions(grid_item.x, grid_item.y, grid_item.grid_scale, tileset, physics, 0, 0)

    if grid_item.flag_for_removal then
      engine:removeEntity(value)
    end

  end
end

function GridItemSystem:requires()
	return {"GridItem", "TileSetGrid"}
end

return GridItemSystem
