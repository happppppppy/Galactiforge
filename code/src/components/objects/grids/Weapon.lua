local Weapon = Component.create("Weapon")
function Weapon:initialize(type, x, y)

  self.base_damage = datasets[type].base_damage
  self.aimed = datasets[type].aimed
  self.activation = datasets[type].activation
  self.fire_rate = datasets[type].fire_rate
  self.fire_time = 0

end