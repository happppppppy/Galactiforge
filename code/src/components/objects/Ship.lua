local Ship = Component.create("Ship")

function Ship:initialize(name)
  self.ship_specs = ship_data[name]
  self.grid_scale = 0.5
  self.grid_size = {
    width = 32,
    height = 32
  }
  self.usable_grid = {}
  self.grid_status = {}
  for row = 1, #self.ship_specs.allowed_grid.grid_map do
    self.grid_status[row] = {}
    for col = 1, #self.ship_specs.allowed_grid.grid_map[row] do
      self.grid_status[row][col] = 0 
    end
  end
end