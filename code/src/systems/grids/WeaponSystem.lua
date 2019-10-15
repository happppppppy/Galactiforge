local TileSetGrid, Bullet = Component.load({"TileSetGrid", "Bullet"})
local PositionPhysics, DynamicPhysics, CollisionPhysics = Component.load({"PositionPhysics", "DynamicPhysics", "CollisionPhysics"})

local WeaponSystem = class("WeaponSystem", System)

function WeaponSystem:fireCannon(event)
  local weapon = event.weapon
  local weapon_component = weapon:get("Weapon")
  local grid_item = weapon:get("GridItem")
  local grid_inventory = weapon:get("GridInventory")
  local tg = weapon:get("TileSetGrid")
  local grid_heat = weapon:get("GridHeat")

  local parent = weapon:getParent()
  local physics = parent:get("PositionPhysics")
  local grid_master = parent:get("GridMaster")

  local modified_fire_rate = weapon_component.fire_rate / ((grid_heat.max_heat - grid_heat.heat)/grid_heat.max_heat)
  if not (weapon_component.fire_time > modified_fire_rate) then weapon_component.fire_time = weapon_component.fire_time + event.dt end

  if weapon_component.fire_time > modified_fire_rate and grid_inventory.resources["shells"].count > 0 then
    if grid_heat.heat < (grid_heat.max_heat - grid_heat.heat_rate) then grid_heat.heat = grid_heat.heat + grid_heat.heat_rate end

    if tg.animated then
      tg.animation_complete = false
    end

    grid_inventory.resources["shells"].count =  grid_inventory.resources["shells"].count - grid_inventory.resources["shells"].use_rate

    local x_vel, y_vel = physics.body:getLinearVelocity()
  
    bullet = Entity()
    local bullet_tile = {}
    bullet_tile.image_ref = 101
    bullet_tile.image_ref_initial_frame = 101
    bullet_tile.image_ref_final_frame = 103
    bullet_tile.animated = true
    bullet_tile.animate_continuous = true
    bullet_tile.render_delay = 0.1
    bullet:add(TileSetGrid(tileset_small, bullet_tile))
    bullet:add(PositionPhysics(world, grid_item.x_pos_grid_physics, grid_item.y_pos_grid_physics, grid_item.t_pos_grid_physics, "dynamic"))
    bullet:add(DynamicPhysics(0, 0, 5000, 5000, 0, 0, 800, x_vel, y_vel))
    bullet:add(CollisionPhysics("Circle",  1, nil, 1, grid_master.physics.category, grid_master.physics.mask))
    bullet:add(Bullet(5, weapon_component.base_damage))
    engine:addEntity(bullet)

    weapon_component.fire_time = 0
  end
end

return WeaponSystem