local grid_functions = require("code/src/systems/grids/grid_functions")

local ThrusterSystem = class("ThrusterSystem", System)

function ThrusterSystem:update(dt)
  for index, value in pairs(self.targets) do
    local thruster = value:get("Thruster")
    local grid_item = value:get("GridItem")
    local grid_inventory = value:get("GridInventory")
    local grid_consumer = value:get("GridConsumer")
    local tileset = value:get("TileSetGrid")

    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")

    local resources_available = grid_functions:getResourceAvailable(grid_inventory)
    local velocity_x, velocity_y = physics.body:getLinearVelocity()

    --Render updates
    thruster.pSystem:setDirection(grid_item.t_render+math.rad(grid_item.direction+90))
    thruster.pSystem:setPosition( grid_item.x_render, grid_item.y_render )
    thruster.pSystem:setColors(255, 255, 255, 30, 255, 255, 255, 0)
    thruster.pSystem:setSpeed(200, 200)
    thruster.pSystem:update(dt)
    
    --Thruster updates
    local fire = false

    if love.keyboard.isDown(thruster.activation) and resources_available and parent:get("PlayerController") ~= nil then
      thruster.particles_active = true
      fire = true

      local magnitude_x = math.sin(physics.body:getAngle()+grid_item.direction_rad)
      local magnitude_y = math.cos(physics.body:getAngle()+grid_item.direction_rad)*-1
      
      local velocity = math.sqrt(velocity_x^2+velocity_y^2)

      physics.body:applyForce(magnitude_x*thruster.base_thrust, magnitude_y*thruster.base_thrust, grid_item.x_render, grid_item.y_render)

    else
      thruster.particles_active = false
    end
    thruster.burn_time = thruster.burn_time + dt
    if fire == true then 
      if thruster.burn_time > thruster.burn_rate and resources_available then
        grid_consumer.count_used = grid_consumer.count_used + 1
        thruster.burn_time = 0
      end
    end
  end
end

function ThrusterSystem:requires()
	return {"Thruster", "GridItem", "GridInventory", "GridConsumer", "TileSetGrid"}
end

return ThrusterSystem