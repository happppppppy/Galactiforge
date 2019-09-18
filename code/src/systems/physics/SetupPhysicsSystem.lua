local SetupPhysicsSystem = class("SetupPhysicsSystem", System)

function SetupPhysicsSystem:onAddEntity(entity, group)
  local dynamics = entity:get("DynamicPhysics")
  local position = entity:get("PositionPhysics")
  local collision = entity:get("CollisionPhysics")
  local sprite = entity:get("Sprite")
  -- if sprite ~= nil then
  --   collision.x = sprite.width
  --   collision.y = sprite.height
  -- end
  
  if dynamics ~= nil  then
    if collision.shape == "Rectangle" then
      position.shape = love.physics.newRectangleShape(collision.x, collision.y)
    elseif collision.shape == "Circle" then
      position.shape = love.physics.newCircleShape(collision.x)
    end

    position.fixture = love.physics.newFixture(position.body, position.shape)
    position.fixture:setCategory(collision.category)
    position.fixture:setUserData(entity)
    if collision.mask ~= nil then
      position.fixture:setMask(collision.mask)
    end
    position.body:setLinearDamping(dynamics.linear_damping)
    position.body:setAngularDamping(dynamics.angular_damping)

    local magnitude_x = math.sin(position.t_start)
    local magnitude_y = math.cos(position.t_start)*-1
    position.body:setLinearVelocity(magnitude_x*dynamics.start_vel + dynamics.x_vel, magnitude_y*dynamics.start_vel + dynamics.y_vel)
  else
    if collision.shape == "Rectangle" then
      position.shape = love.physics.newRectangleShape(collision.x, collision.y)
    elseif collision.shape == "Circle" then
      position.shape = love.physics.newCircleShape(collision.x)
    end
    position.fixture = love.physics.newFixture(position.body, position.shape)
    position.fixture:setCategory(collision.category)
    position.fixture:setUserData(entity)
  end
end

function SetupPhysicsSystem:requires()
	return {"PositionPhysics", "CollisionPhysics"}
end

return SetupPhysicsSystem