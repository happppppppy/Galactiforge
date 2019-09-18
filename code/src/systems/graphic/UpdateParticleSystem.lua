local UpdateParticleSystem = class("UpdateParticleSystem", System)

function UpdateParticleSystem:update(dt)
  for index, value in pairs(self.targets) do
    local particles = value:get("Particles")
    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")

    local t_par = physics.body:getAngle()

    local cos = math.cos(t_par+particles.t_off)
    local sin = math.sin(t_par+particles.t_off)

    local x_loc = physics.body:getX()+particles.d_off*cos
    local y_loc = physics.body:getY()+particles.d_off*sin
    particles.pSystem:setDirection(t_par+math.rad(90))
    particles.pSystem:setPosition( x_loc, y_loc )
    particles.pSystem:setColors(255, 255, 255, 30, 255, 255, 255, 0)
    particles.pSystem:setSpeed(200,500)
    particles.pSystem:update(dt)
  end
end

function UpdateParticleSystem:requires()
	return {"Particles"}
end

return UpdateParticleSystem