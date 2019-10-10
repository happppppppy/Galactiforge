local grid_functions = require("code/src/systems/grids/grid_functions")

local ThrusterSystem = class("ThrusterSystem", System)

function ThrusterSystem:fireEvent(event)
  local value = event.thruster
  local thruster = value:get("Thruster")
  local grid_item = value:get("GridItem")
  local grid_inventory = value:get("GridInventory")

  local parent = value:getParent()
  local physics = parent:get("PositionPhysics")

  local magnitude_x = math.sin(physics.body:getAngle()+grid_item.direction_rad)
  local magnitude_y = math.cos(physics.body:getAngle()+grid_item.direction_rad)*-1

  physics.body:applyForce(magnitude_x*thruster.base_thrust, magnitude_y*thruster.base_thrust, grid_item.x_pos_grid_physics,  grid_item.y_pos_grid_physics)
  
  thruster.burn_time = thruster.burn_time + event.dt
  if thruster.burn_time > thruster.burn_rate and grid_inventory.resources["fuel"].count > 0 then
    grid_inventory.resources["fuel"].count =  grid_inventory.resources["fuel"].count - 1
    thruster.burn_time = 0
  end

end


function ThrusterSystem:update(dt)
  for index, value in pairs(self.targets) do
    local thruster = value:get("Thruster")
    local grid_item = value:get("GridItem")

    --Render updates
    thruster.pSystem:setDirection(grid_item.t_render+math.rad(grid_item.direction+90))
    thruster.pSystem:setPosition(grid_item.x_render, grid_item.y_render )
    thruster.pSystem:setColors(255, 255, 255, 30, 255, 255, 255, 0)
    thruster.pSystem:setSpeed(200, 200)
    thruster.pSystem:update(dt)
  end
end

function ThrusterSystem:requires()
	return {"Thruster", "GridItem"}
end

return ThrusterSystem