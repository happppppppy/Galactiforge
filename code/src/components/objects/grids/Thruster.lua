local Thruster = Component.create("Thruster")
function Thruster:initialize(component_values, direction)

  self.base_thrust = component_values.base_thrust

  --Thruster physics
  self.direction = direction

  if self.direction == 0 then
    self.activation = "w"
  elseif self.direction == 90 then
    self.activation = "a"
  elseif self.direction == 180 then
    self.activation = "s"
  elseif self.direction == 270 then
    self.activation = "d"
  end

  --Particle system
  self.image = love.graphics.newImage(component_values.particle.imagePath)
  self.pSystem = love.graphics.newParticleSystem(self.image, 1000)
  self.pSystem:setParticleLifetime(component_values.particle.lifemin, component_values.particle.lifemax)
  self.pSystem:setEmissionRate(0)
  self.pSystem:setSizeVariation(1)
  self.pSystem:setSizes(
    0.6 * component_values.exhaust_scale, 
    0.6 * component_values.exhaust_scale, 
    1.0 * component_values.exhaust_scale, 
    1.0 * component_values.exhaust_scale)
  self.pSystem:setRotation(component_values.particle.rmin,component_values.particle.rmax)
  self.emitspeed = component_values.particle.emitspeed
  self.particles_active = false

  self.burn_time = 0
  self.burn_rate = component_values.burn_rate

end