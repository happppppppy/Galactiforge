local GridHeat = Component.create("GridHeat")
function GridHeat:initialize()
  self.heat_rate = 20
  self.natural_cool_rate = 40
  self.heat = 1
  self.max_heat  = 100
end