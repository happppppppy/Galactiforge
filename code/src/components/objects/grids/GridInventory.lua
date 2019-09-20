local GridInventory = Component.create("GridInventory")
function GridInventory:initialize(type)

  self.transfer_grid = {}
  for row = 1, #datasets[type].transfer_grid do
    for col = 1, #datasets[type].transfer_grid[row] do
      table.insert(self.transfer_grid, {["usable"] = datasets[type].transfer_grid[row][col], ["grid_reference"]=nil, ["x"] = col-2, ["y"] = row-2})
    end
  end

  self.resource_input = {}
  for i,v in pairs(datasets[type].resource_input) do
    self.resource_input[i] = {}
    self.resource_input[i].count = v.count
    self.resource_input[i].max = v.max
    self.resource_input[i].use_rate = v.use_rate
  end

  self.resource_output = {}
  for i,v in pairs(datasets[type].resource_output) do
    self.resource_output[i] = {}
    self.resource_output[i].count = v.count
    self.resource_output[i].max = v.max
    self.resource_output[i].resource_channel = {}
    self.resource_output[i].available = false
  end

  self.process_delay = datasets[type].process_delay or nil
  self.auto_increment = datasets[type].auto_increment
  self.use_instantaneously = datasets[type].use_instantaneously or false

  self.is_next = false
  self.resources_available = false

  self.next_neighbour = 1
  self.resources_used_count = 0
  self.process_time = 0
  self.grids = {}
end