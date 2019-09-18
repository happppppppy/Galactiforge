local Health = Component.create("Health")

function Health:initialize(health, regeneration_rate)
  self.health = health
  self.regeneration_rate = regeneration_rate
end