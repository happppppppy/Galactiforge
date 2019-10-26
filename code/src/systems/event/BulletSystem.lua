local BulletSystem = class("BulletSystem", System)

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
      local explosion_tile = {}
      explosion_tile.image_ref = 92
      explosion_tile.image_ref_initial_frame = 92
      explosion_tile.image_ref_final_frame = 97
      explosion_tile.animated = true
      explosion_tile.animate_continuous = false
      explosion_tile.render_delay = 0.1
      explosion_tile.onelife = true
      explosion:add(global_components.TileSetGrid(explosion_tile, tileset))
      explosion:add(global_components.PositionPhysics(physics.world, physics.body:getX(), physics.body:getY(), physics.body:getAngle(), "static"))
      engine:addEntity(explosion)
      engine:removeEntity(value)
    end
  end
end

function BulletSystem:requires()
	return {"Bullet", "PositionPhysics"}
end

return BulletSystem