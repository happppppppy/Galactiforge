local RenderParticleSystem = class("RenderParticleSystem", System)

function RenderParticleSystem:draw()
  for index, value in pairs(self.targets) do
    thruster = value:get("Thruster")
    if thruster.pSystem ~= nil then
      if thruster.particles_active then
        thruster.pSystem:setEmissionRate(thruster.emitspeed)
      else
        thruster.pSystem:setEmissionRate(0)
      end
      love.graphics.draw(thruster.pSystem)
    end
  end
end

function RenderParticleSystem:requires()
	return {"Thruster"}
end

return RenderParticleSystem