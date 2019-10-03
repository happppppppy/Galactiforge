local TileSetGrid, Bullet = Component.load({"TileSetGrid", "Bullet"})
local PositionPhysics, DynamicPhysics, CollisionPhysics = Component.load({"PositionPhysics", "DynamicPhysics", "CollisionPhysics"})
local grid_functions = require("code/src/systems/grids/grid_functions")

local WeaponSystem = class("WeaponSystem", System)

function WeaponSystem:update(dt)
  for index, value in pairs(self.targets) do
    local weapon = value:get("Weapon")
    local grid_item = value:get("GridItem")
    local grid_inventory = value:get("GridInventory")
    local tg = value:get("TileSetGrid")
    local grid_consumer = value:get("GridConsumer")
    local grid_heat = value:get("GridHeat")

    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")
    local grid_master = parent:get("GridMaster")

    local fire = false
    local resources_found, resources_available = grid_functions:getResourceAvailable(grid_inventory, "Ammunition")

    if parent:get("PlayerController") ~= nil then
      --Render updates
      if weapon.aimed then
        mouse_x, mouse_y = love.graphics.inverseTransformPoint( love.mouse.getPosition() )
        grid_item.t_pos_grid_physics = math.atan2(mouse_y - grid_item.y_pos_grid_physics, mouse_x - grid_item.x_pos_grid_physics) + math.rad(90)
        grid_item.t_render = grid_item.t_pos_grid_physics --Not correct :(
      end

      --Fire control updates      
      if weapon.activation == "mouse1" then
        if love.mouse.isDown(1) then
          fire = true
        end
      elseif love.keyboard.isDown(weapon.activation) then
        fire = true
      end
        
    elseif parent:get("AIController") ~= nil then
      local AIController = parent:get("AIController")
      local AIFaction = parent:get("Faction")

      for i,v in pairs(global_target_list) do
        if i ~= AIFaction.faction then
          for j,k in pairs(v) do
            grid_item.t_pos_grid_physics = math.atan2(k.grid.y_pos_grid_physics - grid_item.y_pos_grid_physics, k.grid.x_pos_grid_physics - grid_item.x_pos_grid_physics) + 1.5708
            grid_item.t_render = grid_item.t_pos_grid_physics + math.rad(180) --Not correct :(
            AIController.target_direction = grid_item.t_pos_grid_physics
            AIController.target_range = math.sqrt((k.grid.y_pos_grid_physics - grid_item.y_pos_grid_physics)^2 + (k.grid.x_pos_grid_physics - grid_item.x_pos_grid_physics)^2) 
            
            if AIController.target_range < AIController.range then 
              fire = true
              break
            end
          end
        end
      end
    end

    local modified_fire_rate = weapon.fire_rate / ((grid_heat.max_heat - grid_heat.heat)/grid_heat.max_heat)
    if not (weapon.fire_time > modified_fire_rate) then weapon.fire_time = weapon.fire_time + dt end

    if fire then 
      if weapon.fire_time > modified_fire_rate and resources_available then
        if grid_heat.heat < (grid_heat.max_heat - grid_heat.heat_rate) then grid_heat.heat = grid_heat.heat + grid_heat.heat_rate end

        if tg.animated then
          tg.animation_complete = false
        end

        for _,v in pairs(resources_found) do
          grid_inventory.resource_input[v].consumed = grid_inventory.resource_input[v].consumed + 1
        end 

        local x_vel, y_vel = physics.body:getLinearVelocity()
      
        bullet = Entity()
        bullet:add(TileSetGrid(tileset_small, nil, nil, 101, 101, 103, true, true, 0.1, false))
        bullet:add(PositionPhysics(world, grid_item.x_pos_grid_physics, grid_item.y_pos_grid_physics, grid_item.t_pos_grid_physics, "dynamic"))
        bullet:add(DynamicPhysics(0, 0, 5000, 5000, 0, 0, 400, x_vel, y_vel))
        bullet:add(CollisionPhysics("Circle",  1, nil, 1, grid_master.physics.category, grid_master.physics.mask))
        bullet:add(Bullet(5, weapon.base_damage))
        engine:addEntity(bullet)

        weapon.fire_time = 0
      end
    end
  end
end

function WeaponSystem:requires()
	return {"Weapon", "GridItem", "GridInventory", "TileSetGrid", "GridConsumer"}
end

return WeaponSystem