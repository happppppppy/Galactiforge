local GridHeat = Component.create("GridHeat")
function GridHeat:initialize()
  self.heat_rate = 10
  self.natural_cool_rate = 20
  self.heat = 1
  self.max_heat  = 100
end