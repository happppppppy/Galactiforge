local grid_functions = require("code/src/systems/grids/grid_functions")

local GridPhysicsSystem = class("GridPhysicsSystem", System)

function GridPhysicsSystem:onAddEntity(entity)
  local grid_item = entity:get("GridItem")
  local tg = entity:get("TileSetGrid")
  local parent = entity:getParent()
  local physics = parent:get("PositionPhysics")
  local grid_master = parent:get("GridMaster")

  local x_coord = grid_item.x*tg.tileset.tile_width/2
  local y_coord = grid_item.y*tg.tileset.tile_height/2
  local width = tg.tileset.tile_width * grid_item.grid_scale
  local height = tg.tileset.tile_height * grid_item.grid_scale
  local angle = grid_item.t_render

  local shape = love.physics.newRectangleShape(x_coord, y_coord, width, height, angle)
  grid_item.fixture = love.physics.newFixture(physics.body, shape, 20)
  grid_item.fixture:setCategory(grid_master.physics.category)
  grid_item.fixture:setMask(grid_master.physics.mask)
  grid_item.fixture:setUserData(entity)

  physics.body:setLinearDamping(grid_master.physics.linear_damping)
  physics.body:setAngularDamping(grid_master.physics.angular_damping)

end

function GridPhysicsSystem:requires()
	return {"GridPhysics", "GridItem", "TileSetGrid"}
end

return GridPhysicsSystem
