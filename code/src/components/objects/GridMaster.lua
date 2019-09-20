local GridMaster = Component.create("GridMaster")

function GridMaster:initialize(grid, grid_specs, grid_scale, grid_width, grid_height, category, mask)
  self.grid_specs = grid_specs
  self.grid = grid or {}
  self.grid_scale = grid_scale
  self.grid_size = {
    width = grid_width,
    height = grid_height
  }
  self.usable_grid = {}
  self.grid_status = {}
  self.physics = {
    category = category,
    mask = mask,
    linear_damping= 0.3,
    angular_damping = 0.5
  }
  for row = 1, #self.grid_specs.allowed_grid.grid_map do
    self.grid_status[row] = {}
    for col = 1, #self.grid_specs.allowed_grid.grid_map[row] do
      self.grid_status[row][col] = 0 
    end
  end

end