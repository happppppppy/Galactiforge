local Particles = Component.create("Particles")

function Particles:initialize(imagePath, lifemin, lifemax, speed, rmin, rmax, d_off, t_off)
  self.image = love.graphics.newImage(imagePath)
  self.pSystem = love.graphics.newParticleSystem(self.image, 1000)
  self.pSystem:setParticleLifetime(lifemin, lifemax)
  self.pSystem:setEmissionRate(0)
  self.pSystem:setSizeVariation(1)
  self.pSystem:setSizes(1, 1.2, 1.4, 0.6)
  self.pSystem:setRotation(rmin,rmax)
  self.rmin = rmin
  self.rmax = rmax
  self.emitbool = false
  self.emitspeed = speed
  self.d_off = d_off
  self.t_off = t_off
end