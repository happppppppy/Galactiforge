local GridHeat = Component.create("GridHeat")
function GridHeat:initialize(component_values)
  --Config values
  self.heat_rate = component_values.heat_rate
  self.natural_cool_rate = component_values.natural_cool_rate
  self.max_heat  = component_values.max_heat
  
  --Dynamic values
  self.heat = 1
end