local Thruster = Component.create("Thruster")
function Thruster:initialize(type, x, y, direction)

  self.base_thrust = datasets[type].base_thrust

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
  self.image = love.graphics.newImage(datasets[type].particle.imagePath)
  self.pSystem = love.graphics.newParticleSystem(self.image, 1000)
  self.pSystem:setParticleLifetime(datasets[type].particle.lifemin, datasets[type].particle.lifemax)
  self.pSystem:setEmissionRate(0)
  self.pSystem:setSizeVariation(1)
  self.pSystem:setSizes(0.6, 0.6, 1.0, 1.0)
  self.pSystem:setRotation(datasets[type].particle.rmin,datasets[type].particle.rmax)
  self.emitspeed = datasets[type].particle.emitspeed
  self.particles_active = false

  self.burn_time = 0
  self.burn_rate = datasets[type].burn_rate

end