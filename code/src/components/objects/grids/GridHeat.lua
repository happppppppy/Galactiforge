local GridHeat = Component.create("GridHeat")
function GridHeat:initialize(heat_rate, natural_cool_rate, max_heat)
  --Config values
  self.heat_rate = heat_rate
  self.natural_cool_rate = natural_cool_rate
  self.max_heat  = max_heat
  
  --Dynamic values
  self.heat = 1
end