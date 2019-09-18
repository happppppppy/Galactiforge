local GridItem = Component.create("GridItem")
function GridItem:initialize(type, x, y, category, auto_increment, direction)

  self.category = category or nil
  self.type = type
  self.transfer_grid = {}
  for i,v in pairs(datasets[type].transfer_grid) do
      table.insert(self.transfer_grid, v)
  end

  self.process_delay = datasets[type].process_delay or nil

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
    self.resource_output[i].next_output = 1
  end
  
  self.image_ref = datasets[type].image_ref or nil
  self.grid_scale = 0.5
  self.process_time = 0
  self.x = x
  self.y = y
  
  self.is_next = false
  self.next_neighbour = 1
  self.neighbours = {}
  self.neighbour_grid = {}
  self.grids = {}

  self.x_render = 0
  self.y_render = 0
  self.t_render = 0

  self.direction = nil or direction

  if self.direction == 0 then
    self.activation = "up"
  elseif self.direction == 90 then
    self.activation = "left"
  elseif self.direction == 180 then
    self.activation = "down"
  elseif self.direction == 270 then
    self.activation = "right"
  end

  self.auto_increment = auto_increment
  self.use_instantaneously = datasets[type].use_instantaneously or false
  self.resources_used_count = 0

  self.resources_available = false

  self.fixture = nil
end