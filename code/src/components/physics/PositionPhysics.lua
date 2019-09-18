local PositionPhysics = Component.create("PositionPhysics")

function PositionPhysics:initialize(world, x_start, y_start, t_start, type)
  self.world = world
  self.body = love.physics.newBody(world, x_start, y_start, type) --remember, the shape (the rectangle we create next) anchors to the body from its center, so we have to move it to (650/2, 650-50/2)
  self.body:setAngle(t_start)
  self.x_start = x_start
  self.y_start = y_start
  self.t_start = t_start

end