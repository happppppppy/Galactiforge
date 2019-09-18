local Bullet = Component.create("Bullet")

function Bullet:initialize(lifespan, damage)
  self.lifespan = lifespan
  self.lifetime = 0
  self.impact = false
  self.damage = damage

end