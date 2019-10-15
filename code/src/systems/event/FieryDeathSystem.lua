local TileSetGrid, PositionPhysics = Component.load({"TileSetGrid", "PositionPhysics"})

local FieryDeathSystem = class("FieryDeathSystem", System)

local function firey_death(physics, stats, health, width, height, dt, value)
  if health.health <= 0 then
    stats.elapsed_time = stats.elapsed_time + dt
    if stats.elapsed_time > stats.animation_delay and stats.count < stats.max_count then
      stats.count = stats.count + 1
      explosion = Entity()
      explosion:add(TileSetGrid(tileset_small, nil, nil, 91, 91, 97, true, false, 0.1, true))
      explosion:add(PositionPhysics(physics.world, physics.body:getX() + math.random (width/2*-1, width/2), physics.body:getY() + math.random (height/2*-1, height/2), physics.body:getAngle() + math.random (), "static"))
      engine:addEntity(explosion)
      stats.elapsed_time = 0
    end
    if stats.count >= stats.max_count then
      engine:removeEntity(value, true)
    end
  end
end

function FieryDeathSystem:update(dt)
  for index, value in pairs(self.targets.pool1) do
    local physics = value:get("PositionPhysics")
    local stats = value:get("FieryDeath")
    local health = value:get("Health")
    local sprite = value:get("Sprite")

    firey_death(physics, stats, health, sprite.width, sprite.height, dt, value)
  end

  for index, value in pairs(self.targets.pool2) do
    local grid_item = value:get("GridItem")
    local tg = value:get("TileSetGrid")
    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")
    local stats = value:get("FieryDeath")
    local health = value:get("Health")

    local width = tg.tileset.tile_width
    local height = tg.tileset.tile_height

    if health.health <= 0 then


      stats.elapsed_time = stats.elapsed_time + dt
      if stats.elapsed_time > stats.animation_delay and stats.count < stats.max_count then
        stats.count = stats.count + 1
        explosion = Entity()
        explosion:add(TileSetGrid(tileset_small, nil, nil, 91, 91, 97, true, false, 0.1, true))
        explosion:add(PositionPhysics(physics.world, grid_item.x_pos_grid_physics + math.random (width/2*-1, width/2), grid_item.y_pos_grid_physics + math.random (height/2*-1, height/2), grid_item.t_pos_grid_physics + math.random (), "static"))
        engine:addEntity(explosion)
        stats.elapsed_time = 0
      end
      
      if stats.count >= stats.max_count then
        engine:removeEntity(value, true)
      end
      
      -- If the ship core is destroyed, then the entire ship is destroyed.
      if grid_item.x == 0 and grid_item.y == 0 then
        engine:removeEntity(parent, true)
      end

    end
  end
end

function FieryDeathSystem:requires()
	return {pool1 = {"FieryDeath", "PositionPhysics", "Health"}, pool2={"FieryDeath", "GridItem", "Health"}}
end

return FieryDeathSystem