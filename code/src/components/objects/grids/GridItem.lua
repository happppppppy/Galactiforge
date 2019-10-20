local GridItem = Component.create("GridItem")
function GridItem:initialize(type, x, y, direction, grid_scale, component_name)
  self.type = type
  self.grid_scale = grid_scale

  self.image_ref = datasets[type].image_ref or nil
  self.x = x
  self.y = y

  self.x_render = 0
  self.y_render = 0
  self.t_render = 0

  self.t_pos_grid_physics = 0
  self.x_pos_grid_physics = 0
  self.y_pos_grid_physics = 0

  self.direction = direction

  if self.direction == 0 then
    self.activation = "up"
  elseif self.direction == 90 then
    self.activation = "left"
  elseif self.direction == 180 then
    self.activation = "down"
  elseif self.direction == 270 then
    self.activation = "right"
  end

  self.direction_rad = math.rad(self.direction)

  self.fixture = nil

  self.buildable = datasets[type].buildable or true
  
end