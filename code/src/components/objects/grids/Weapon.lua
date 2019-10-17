local Weapon = Component.create("Weapon")
function Weapon:initialize(component_values)

  self.base_damage = component_values.base_damage
  self.aimed = component_values.aimed
  self.activation = component_values.activation
  self.fire_rate = component_values.fire_rate
  self.projectile_velocity = component_values.projectile_velocity
  self.fire_time = 0

end