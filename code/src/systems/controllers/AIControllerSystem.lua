local AIControllerSystem = class("AIControllerSystem", System)

function AIControllerSystem:update(dt)
  if global_ai_active then
    for index, value in pairs(self.targets) do
      physics = value:get("PositionPhysics")
      AIController = value:get("AIController")

      if AIController.target_direction ~= physics.body:getAngle() then

        if AIController.target_direction > physics.body:getAngle() then
          physics.body:applyTorque(300000)
        elseif AIController.target_direction < physics.body:getAngle() then
          physics.body:applyTorque(-300000)
        end
      end

      if AIController.target_range > AIController.range then
        local magnitude_x = math.sin(physics.body:getAngle())
        local magnitude_y = math.cos(physics.body:getAngle())*-1
        local force = 10000
        physics.body:applyForce(magnitude_x * force, magnitude_y * force)
      end
    end
  end
end

function AIControllerSystem:requires()
	return {"AIController"}
end

return AIControllerSystem