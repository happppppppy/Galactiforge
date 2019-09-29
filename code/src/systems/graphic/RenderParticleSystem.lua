local RenderParticleSystem = class("RenderParticleSystem", System)

function RenderParticleSystem:draw()
  for index, value in pairs(self.targets) do
    local thruster = value:get("Thruster")
    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")
    if thruster.pSystem ~= nil then
      if thruster.particles_active then
        thruster.pSystem:setEmissionRate(thruster.emitspeed)
      else
        thruster.pSystem:setEmissionRate(0)
      end
      love.graphics.push()
      love.graphics.translate(physics.body:getX(),physics.body:getY())
      love.graphics.rotate(physics.body:getAngle())

      love.graphics.draw(thruster.pSystem)

      love.graphics.pop()

    end
  end
end

function RenderParticleSystem:requires()
	return {"Thruster"}
end

return RenderParticleSystem