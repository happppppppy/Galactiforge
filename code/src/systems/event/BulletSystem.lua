local BulletSystem = class("BulletSystem", System)
local TileSetGrid, PositionPhysics = Component.load({"TileSetGrid", "PositionPhysics"})

function BulletSystem:update(dt)
  for index, value in pairs(self.targets) do
    local bullet = value:get("Bullet")
    local physics = value:get("PositionPhysics")
    bullet.lifetime = bullet.lifetime + dt
    if bullet.lifetime >= bullet.lifespan then
      engine:removeEntity(value)
    end

    if bullet.impact then
      explosion = Entity()
      explosion:add(TileSetGrid(tileset, nil, nil, 92, 92, 97, true, false, 0.1, true))
      explosion:add(PositionPhysics(physics.world, physics.body:getX(), physics.body:getY(), physics.body:getAngle(), "static"))
      engine:addEntity(explosion)
      engine:removeEntity(value)
    end
  end
end

function BulletSystem:requires()
	return {"Bullet", "PositionPhysics"}
end

return BulletSystem