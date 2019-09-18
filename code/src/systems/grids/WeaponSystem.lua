local TileSetGrid, Bullet = Component.load({"TileSetGrid", "Bullet"})
local PositionPhysics, DynamicPhysics, CollisionPhysics = Component.load({"PositionPhysics", "DynamicPhysics", "CollisionPhysics"})
local grid_functions = require("code/src/systems/grids/grid_functions")

local WeaponSystem = class("WeaponSystem", System)

function WeaponSystem:update(dt)
  for index, value in pairs(self.targets) do
    local weapon = value:get("Weapon")
    local grid_item = value:get("GridItem")
    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")
    local collision = parent:get("CollisionPhysics")
    local tg = value:get("TileSetGrid")

    
    local fire = false
    local resources_available = grid_functions:getResourceAvailable(grid_item)
    if parent:get("PlayerController") ~= nil then
      --Render updates
      if weapon.aimed then
        mouse_x, mouse_y = love.graphics.inverseTransformPoint( love.mouse.getPosition() )
        grid_item.t_render = math.atan2(mouse_y - grid_item.y_render, mouse_x - grid_item.x_render) + math.rad(90)
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
      local targetable_grids = engine:getEntitiesWithComponent("GridItem")

      for i,v in pairs(targetable_grids) do
        --Get angle to target
        if v:getParent() ~= parent then
          local target_grid = v:get("GridItem")
          local parent = v:getParent()
          local faction = parent:get("Faction")
          
          if faction.faction ~= AIFaction.faction then
            grid_item.t_render = math.atan2(target_grid.y_render - grid_item.y_render, target_grid.x_render - grid_item.x_render) + math.rad(90)
            AIController.target_direction = grid_item.t_render
            AIController.target_range = math.sqrt((target_grid.y_render - grid_item.y_render)^2 + (target_grid.x_render - grid_item.x_render)^2) 
            
            if AIController.target_range < AIController.range then 
              fire = true
              break
            end
          end
        end
      end

      --See if target is in range and set fire to true
      
    end

      if not (weapon.fire_time > weapon.fire_rate) then weapon.fire_time = weapon.fire_time + dt end

      if fire then 
        if weapon.fire_time > weapon.fire_rate and resources_available then
          if tg.animated then
            tg.animation_complete = false
          end

          grid_item.resources_used_count = grid_item.resources_used_count + 1

          local x_vel, y_vel = physics.body:getLinearVelocity()
          
          _, x_render, y_render = grid_functions:getRenderPositions(grid_item.x, grid_item.y, grid_item.grid_scale, tg, physics, 0, 0)
          
          bullet = Entity()
          bullet:add(TileSetGrid(tileset_small, nil, nil, 101, 101, 103, true, true, 0.1, false))
          bullet:add(PositionPhysics(world, x_render, y_render, grid_item.t_render, "dynamic"))
          bullet:add(DynamicPhysics(0, 0, 5000, 5000, 0, 0, 400, x_vel, y_vel))
          bullet:add(CollisionPhysics("Circle",  1, nil, 1, collision.category, collision.mask))
          bullet:add(Bullet(5, weapon.base_damage))
          engine:addEntity(bullet)

          weapon.fire_time = 0
        end
      end


  end
end

function WeaponSystem:requires()
	return {"Weapon"}
end

return WeaponSystem