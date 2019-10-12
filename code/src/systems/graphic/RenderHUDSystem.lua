local RenderHUDSystem = class("RenderHUDSystem", System)

function RenderHUDSystem:draw()
  for index, value in pairs(self.targets) do
    local physics = value:get("PositionPhysics")
    local health = value:get("Health")
    local grid = value:get("Grid")
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
end

function RenderHUDSystem:requires()
	return {"GridMaster", "Health", "PositionPhysics", "PlayerController"}
end

return RenderHUDSystem