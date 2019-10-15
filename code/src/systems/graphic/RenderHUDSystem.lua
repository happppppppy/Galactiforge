local RenderHUDSystem = class("RenderHUDSystem", System)

function RenderHUDSystem:draw()
  for index, value in pairs(self.targets.pool1) do
    local physics = value:get("PositionPhysics")
    local grid_master = value:get("GridMaster")

    local x_vel, y_vel = physics.body:getLinearVelocity( )
    local ship_velocity = math.sqrt(x_vel^2 + y_vel^2)
    love.graphics.setFont( game_font )

    love.graphics.push()
    love.graphics.origin()

    local x_loc = 20
    local y_loc = love.graphics.getHeight() - 20

    love.graphics.draw(tileset_small.image, tileset_small.tiles[datasets[global_component_name_list[global_component_index]].image_ref], x_loc + tileset.tile_width/2, y_loc-160 + tileset.tile_height/2, math.rad(global_component_directions[global_component_direction_index]), 1, 1, tileset.tile_width/2, tileset.tile_height/2) 
    love.graphics.print("Selected: "..global_component_name_list[global_component_index], x_loc, y_loc-120, 0, 1, 1)
    love.graphics.print(string.format("Ship Velocity: %.2f ", ship_velocity), x_loc, y_loc-100, 0, 1, 1)
    love.graphics.print(string.format("X Coordinate: %.2f ", physics.body:getX()), x_loc, y_loc-80, 0, 1, 1)
    love.graphics.print(string.format("Y Coordinate: %.2f ", physics.body:getY()), x_loc, y_loc-60, 0, 1, 1)
    love.graphics.print(string.format("Ship Mass: %.2f ", physics.body:getMass()), x_loc, y_loc-20, 0, 1, 1)
    love.graphics.print(string.format("Ship Health: %d ", grid_master.health), x_loc, y_loc, 0, 1, 1)
    love.graphics.print(string.format("FPS: %d ", love.timer.getFPS()), x_loc, y_loc-180, 0, 1, 1)
    love.graphics.print(string.format("DT: %d ", love.timer.getAverageDelta()), x_loc, y_loc-200, 0, 1, 1)
  
    love.graphics.pop()
  end

  for index, value in pairs(self.targets.pool2) do
    local playership = engine:getEntitiesWithComponent("PositionPhysics")
    local playership_physics = 0
    for i,v in pairs(playership) do
      if v:get("PlayerController") ~= nil then
        playership_physics = v:get("PositionPhysics")
        break
      end
    end

    local aiship_physics = value:get("PositionPhysics")
    local range_limit = 2000
    transparency_limit = 0.6
    local t_pos_grid_physics = math.atan2(aiship_physics.body:getY() - playership_physics.body:getY(), aiship_physics.body:getX() - playership_physics.body:getX())
    
    local range = math.sqrt((aiship_physics.body:getY() - playership_physics.body:getY())^2 + (aiship_physics.body:getX() - playership_physics.body:getX())^2)
    local distance = 0
    if range > love.graphics.getHeight()/2 - 100 then
      distance = love.graphics.getHeight()/2 - 100
    else
      distance = range
    end
    
    local x = distance * math.cos(t_pos_grid_physics) + playership_physics.body:getX()
    local y = distance * math.sin(t_pos_grid_physics) + playership_physics.body:getY()
    
    local r,g,b,a = love.graphics.getColor()
    local transparency = 0
    if range < range_limit then transparency = 1 / (range/300) end
    if transparency > transparency_limit then transparency = transparency_limit end
    love.graphics.setColor(1,0,0,transparency)
    love.graphics.circle('fill',x,y,10)
    -- love.graphics.line(playership_physics.body:getX(), playership_physics.body:getY(), x, y)--aiship_physics.body:getX(), aiship_physics.body:getY())
    love.graphics.setColor(r,g,b,a)
  end
end

function RenderHUDSystem:requires()
	return {pool1 = {"GridMaster", "Health", "PositionPhysics", "PlayerController"}, pool2 = {"GridMaster", "PositionPhysics", "AIController"}}
end

return RenderHUDSystem