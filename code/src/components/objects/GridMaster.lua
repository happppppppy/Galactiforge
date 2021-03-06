local GridMaster = Component.create("GridMaster")

function GridMaster:initialize(grid, grid_specs, grid_scale, grid_width, grid_height, category, mask, player)
  self.player = player or false

  self.grid_specs = grid_specs
  self.grid = grid or {}
  self.grid_scale = grid_scale
  self.grid_size = {
    width = grid_width,
    height = grid_height
  }
  self.usable_grid = {}
  self.physics = {
    category = category,
    mask = mask,
    linear_damping= 0.3,
    angular_damping = 0.5
  }

  self.grid_items = {}
  if #self.grid_specs.allowed_grid.grid_map ~= 0 then
    for row = 1, #self.grid_specs.allowed_grid.grid_map do
      self.grid_items[row] = {}
      for col = 1, #self.grid_specs.allowed_grid.grid_map[row] do
        self.grid_items[row][col] = 0
      end
    end

  else

    local i=1
    for row = 1, self.grid_specs.allowed_grid.grid_origin.x*2-1, 1 do
      self.grid_items[row] = {}
      self.grid_specs.allowed_grid.grid_map[row] = {}
      for col = 1, self.grid_specs.allowed_grid.grid_origin.x*2-1, 1 do
        self.grid_specs.allowed_grid.grid_map[row][col] = 1
        self.grid_items[row][col] = 0
      end
    end
  end

  self.grids = {}
  
end